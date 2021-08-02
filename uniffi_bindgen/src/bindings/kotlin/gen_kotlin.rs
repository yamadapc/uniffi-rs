/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, MixedCase, ShoutySnakeCase};
use serde::{Deserialize, Serialize};

use crate::interface::*;
use crate::MergeWith;

use crate::bindings::backend::{ CodeType, TypeIdentifier, TypeOracle };

mod enums;

// Some config options for it the caller wants to customize the generated Kotlin.
// Note that this can only be used to control details of the Kotlin *that do not affect the underlying component*,
// sine the details of the underlying component are entirely determined by the `ComponentInterface`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Config {
    package_name: Option<String>,
    cdylib_name: Option<String>,
}

impl Config {
    pub fn package_name(&self) -> String {
        if let Some(package_name) = &self.package_name {
            package_name.clone()
        } else {
            "uniffi".into()
        }
    }

    pub fn cdylib_name(&self) -> String {
        if let Some(cdylib_name) = &self.cdylib_name {
            cdylib_name.clone()
        } else {
            "uniffi".into()
        }
    }
}

impl From<&ComponentInterface> for Config {
    fn from(ci: &ComponentInterface) -> Self {
        Config {
            package_name: Some(format!("uniffi.{}", ci.namespace())),
            cdylib_name: Some(format!("uniffi_{}", ci.namespace())),
        }
    }
}

impl MergeWith for Config {
    fn merge_with(&self, other: &Self) -> Self {
        Config {
            package_name: self.package_name.merge_with(&other.package_name),
            cdylib_name: self.cdylib_name.merge_with(&other.cdylib_name),
        }
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "wrapper.kt")]
pub struct KotlinWrapper<'a> {
    config: Config,
    ci: &'a ComponentInterface,
}
impl<'a> KotlinWrapper<'a> {
    pub fn new(config: Config, ci: &'a ComponentInterface) -> Self {
        Self { config, ci }
    }
}

struct FallbackCodeType {
    type_: TypeIdentifier,
}

impl FallbackCodeType {
    fn new(type_: TypeIdentifier) -> Self { Self { type_ } }

    fn type_identifier(&self, _oracle: &dyn TypeOracle) -> &TypeIdentifier {
        &self.type_
    }
}

impl CodeType for FallbackCodeType {
    fn type_label(&self, oracle: &dyn TypeOracle) -> Result<String, askama::Error> {
        let type_ = self.type_identifier(oracle);
        legacy_kt::type_kt(type_)
    }

    fn canonical_name(&self, oracle: &dyn TypeOracle) -> Result<String, askama::Error> {
        let type_ = self.type_identifier(oracle);
        Ok(type_.canonical_name())
    }

    fn literal(&self, _oracle: &dyn TypeOracle, literal: &Literal) -> Result<String, askama::Error> {
        legacy_kt::literal_kt(literal)
    }

    fn lower(&self, oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        legacy_kt::lower_kt(nm, self.type_identifier(oracle))
    }

    fn write(&self,
        oracle: &dyn TypeOracle,
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
    ) -> Result<String, askama::Error> {
        legacy_kt::write_kt(nm, target, self.type_identifier(oracle))
    }

    fn lift(&self, oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        legacy_kt::lift_kt(nm, self.type_identifier(oracle))
    }

    fn read(&self, oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        legacy_kt::read_kt(nm, self.type_identifier(oracle))
    }
}

#[derive(Default)]
pub struct KotlinTypeOracle;

impl KotlinTypeOracle {
    fn create_code_type(&self, type_: TypeIdentifier) -> Box<dyn CodeType> {
        match type_ {
            Type::Enum(id) => Box::new(enums::EnumCodeType::new(id)),
            _ => Box::new(FallbackCodeType::new(type_)),
        }
    }
}

impl TypeOracle for KotlinTypeOracle {
    fn find(&self, type_: &TypeIdentifier) -> Result<Box<dyn CodeType>, askama::Error> {
        Ok(
            self.create_code_type(type_.clone())
        )
    }

    /// Get the idiomatic Kotlin rendering of a class name (for enums, records, errors, etc).
    fn class_name(&self, nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    /// Get the idiomatic Kotlin rendering of a function name.
    fn fn_name(&self, nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    /// Get the idiomatic Kotlin rendering of a variable name.
    fn var_name(&self, nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    /// Get the idiomatic Kotlin rendering of an individual enum variant.
    fn enum_variant(&self, nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_shouty_snake_case())
    }

    /// Get the idiomatic Kotlin rendering of an exception name
    ///
    /// This replaces "Error" at the end of the name with "Exception".  Rust code typically uses
    /// "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
    /// and is distinguished from an "Exception".
    fn exception_name(&self, nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        let name = nm.to_string();
        match name.strip_suffix("Error") {
            None => Ok(name),
            Some(stripped) => {
                let mut kt_exc_name = stripped.to_owned();
                kt_exc_name.push_str("Exception");
                Ok(kt_exc_name)
            }
        }
    }
}

mod filters {
    use super::*;
    use std::fmt;

    fn oracle() -> impl TypeOracle {
        KotlinTypeOracle
    }

    pub fn type_kt(type_: &Type) -> Result<String, askama::Error> {
        let oracle = oracle();
        oracle.find(type_)?.type_label(&oracle)
    }

    pub fn lower_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let oracle = oracle();
        oracle.find(type_)?.lower(&oracle, nm)
    }

    pub fn write_kt(
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
        type_: &Type,
    ) -> Result<String, askama::Error> {
        let oracle = oracle();
        oracle.find(type_)?.write(&oracle, nm, target)
    }

    pub fn lift_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let oracle = oracle();
        oracle.find(type_)?.lift(&oracle, nm)
    }

    pub fn literal_kt(literal: &Literal) -> Result<String, askama::Error> {
        let type_ = match literal {
            Literal::Enum(_, type_) => type_,
            Literal::Int(_, _, type_) => type_,
            Literal::UInt(_, _, type_) => type_,
            Literal::Float(_, type_) => type_,
            _ => return legacy_kt::literal_kt(literal),
        };

        let oracle = oracle();
        oracle.find(type_)?.literal(&oracle, literal)
    }

    pub fn read_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let oracle = oracle();
        oracle.find(type_)?.read(&oracle, nm)
    }

    /// Get the Kotlin syntax for representing a given low-level `FFIType`.
    pub fn type_ffi(type_: &FFIType) -> Result<String, askama::Error> {
        Ok(match type_ {
            // Note that unsigned integers in Kotlin are currently experimental, but java.nio.ByteBuffer does not
            // support them yet. Thus, we use the signed variants to represent both signed and unsigned
            // types from the component API.
            FFIType::Int8 | FFIType::UInt8 => "Byte".to_string(),
            FFIType::Int16 | FFIType::UInt16 => "Short".to_string(),
            FFIType::Int32 | FFIType::UInt32 => "Int".to_string(),
            FFIType::Int64 | FFIType::UInt64 => "Long".to_string(),
            FFIType::Float32 => "Float".to_string(),
            FFIType::Float64 => "Double".to_string(),
            FFIType::RustArcPtr => "Pointer".to_string(),
            FFIType::RustBuffer => "RustBuffer.ByValue".to_string(),
            FFIType::ForeignBytes => "ForeignBytes.ByValue".to_string(),
            FFIType::ForeignCallback => "ForeignCallback".to_string(),
        })
    }

    /// Get the idiomatic Kotlin rendering of a class name (for enums, records, errors, etc).
    pub fn class_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        oracle().class_name(nm)
    }

    /// Get the idiomatic Kotlin rendering of a function name.
    pub fn fn_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        oracle().fn_name(nm)
    }

    /// Get the idiomatic Kotlin rendering of a variable name.
    pub fn var_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        oracle().var_name(nm)
    }

    /// Get the idiomatic Kotlin rendering of an individual enum variant.
    pub fn enum_variant_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        oracle().enum_variant(nm)
    }

    /// Get the idiomatic Kotlin rendering of an exception name
    ///
    /// This replaces "Error" at the end of the name with "Exception".  Rust code typically uses
    /// "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
    /// and is distinguished from an "Exception".
    pub fn exception_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        oracle().exception_name(nm)
    }
}

mod legacy_kt {
    use super::*;
    use std::fmt;

    /**
     * Temporarily put the formatters here.
     */

    /// Get the idiomatic Kotlin rendering of a class name (for enums, records, errors, etc).
    pub fn class_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_camel_case())
    }

    /// Get the idiomatic Kotlin rendering of a variable name.
    pub fn var_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_mixed_case())
    }

    /// Get the idiomatic Kotlin rendering of an individual enum variant.
    pub fn enum_variant_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(nm.to_string().to_shouty_snake_case())
    }


    /// Get the Kotlin syntax for representing a given api-level `Type`.
    pub fn type_kt(type_: &Type) -> Result<String, askama::Error> {
        Ok(match type_ {
            // These native Kotlin types map nicely to the FFI without conversion.
            Type::UInt8 => "UByte".to_string(),
            Type::UInt16 => "UShort".to_string(),
            Type::UInt32 => "UInt".to_string(),
            Type::UInt64 => "ULong".to_string(),
            Type::Int8 => "Byte".to_string(),
            Type::Int16 => "Short".to_string(),
            Type::Int32 => "Int".to_string(),
            Type::Int64 => "Long".to_string(),
            Type::Float32 => "Float".to_string(),
            Type::Float64 => "Double".to_string(),
            // These types need conversion, and special handling for lifting/lowering.
            Type::Boolean => "Boolean".to_string(),
            Type::String => "String".to_string(),
            Type::Timestamp => "java.time.Instant".to_string(),
            Type::Duration => "java.time.Duration".to_string(),
            Type::Enum(name)
            | Type::Record(name)
            | Type::Object(name)
            | Type::Error(name)
            | Type::CallbackInterface(name) => class_name_kt(name)?,
            Type::Optional(t) => format!("{}?", type_kt(t)?),
            Type::Sequence(t) => format!("List<{}>", type_kt(t)?),
            Type::Map(t) => format!("Map<String, {}>", type_kt(t)?),
        })
    }

    /// Get a Kotlin expression for lowering a value into something we can pass over the FFI.
    ///
    /// Where possible, this delegates to a `lower()` method on the type itself, but special
    /// handling is required for some compound data types.
    pub fn lower_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let nm = var_name_kt(nm)?;
        Ok(match type_ {
            Type::CallbackInterface(_) => format!(
                "{}Internals.lower({})",
                class_name_kt(&type_.canonical_name())?,
                nm,
            ),
            Type::Optional(_)
            | Type::Sequence(_)
            | Type::Map(_)
            | Type::Timestamp
            | Type::Duration => {
                format!("lower{}({})", class_name_kt(&type_.canonical_name())?, nm,)
            }
            _ => format!("{}.lower()", nm),
        })
    }

    /// Get a Kotlin expression for writing a value into a byte buffer.
    ///
    /// Where possible, this delegates to a `write()` method on the type itself, but special
    /// handling is required for some compound data types.
    pub fn write_kt(
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
        type_: &Type,
    ) -> Result<String, askama::Error> {
        let nm = var_name_kt(nm)?;
        Ok(match type_ {
            Type::CallbackInterface(_) => format!(
                "{}Internals.write({}, {})",
                class_name_kt(&type_.canonical_name())?,
                nm,
                target,
            ),
            Type::Optional(_)
            | Type::Sequence(_)
            | Type::Map(_)
            | Type::Timestamp
            | Type::Duration => format!(
                "write{}({}, {})",
                class_name_kt(&type_.canonical_name())?,
                nm,
                target,
            ),
            _ => format!("{}.write({})", nm, target),
        })
    }

    /// Get a Kotlin expression for lifting a value from something we received over the FFI.
    ///
    /// Where possible, this delegates to a `lift()` method on the type itself, but special
    /// handling is required for some compound data types.
    pub fn lift_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            Type::CallbackInterface(_) => format!(
                "{}Internals.lift({})",
                class_name_kt(&type_.canonical_name())?,
                nm,
            ),
            Type::Optional(_)
            | Type::Sequence(_)
            | Type::Map(_)
            | Type::Timestamp
            | Type::Duration => format!("lift{}({})", class_name_kt(&type_.canonical_name())?, nm),
            _ => format!("{}.lift({})", type_kt(type_)?, nm),
        })
    }



    /// Get a Kotlin expression for reading a value from a byte buffer.
    ///
    /// Where possible, this delegates to a `read()` method on the type itself, but special
    /// handling is required for some compound data types.
    pub fn read_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let nm = nm.to_string();
        Ok(match type_ {
            Type::CallbackInterface(_) => format!(
                "{}Internals.read({})",
                class_name_kt(&type_.canonical_name())?,
                nm,
            ),
            Type::Optional(_)
            | Type::Sequence(_)
            | Type::Map(_)
            | Type::Timestamp
            | Type::Duration => format!("read{}({})", class_name_kt(&type_.canonical_name())?, nm),
            _ => format!("{}.read({})", type_kt(type_)?, nm),
        })
    }

    pub fn literal_kt(literal: &Literal) -> Result<String, askama::Error> {
        fn typed_number(type_: &Type, num_str: String) -> Result<String, askama::Error> {
            Ok(match type_ {
                // Bytes, Shorts and Ints can all be inferred from the type.
                Type::Int8 | Type::Int16 | Type::Int32 => num_str,
                Type::Int64 => format!("{}L", num_str),

                Type::UInt8 | Type::UInt16 | Type::UInt32 => format!("{}u", num_str),
                Type::UInt64 => format!("{}uL", num_str),

                Type::Float32 => format!("{}f", num_str),
                Type::Float64 => num_str,
                _ => panic!("Unexpected literal: {} is not a number", num_str),
            })
        }

        Ok(match literal {
            Literal::Boolean(v) => format!("{}", v),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Null => "null".into(),
            Literal::EmptySequence => "listOf()".into(),
            Literal::EmptyMap => "mapOf".into(),
            Literal::Enum(v, type_) => format!("{}.{}", type_kt(type_)?, enum_variant_kt(v)?),
            Literal::Int(i, radix, type_) => typed_number(
                type_,
                match radix {
                    Radix::Octal => format!("{:#x}", i),
                    Radix::Decimal => format!("{}", i),
                    Radix::Hexadecimal => format!("{:#x}", i),
                },
            )?,
            Literal::UInt(i, radix, type_) => typed_number(
                type_,
                match radix {
                    Radix::Octal => format!("{:#x}", i),
                    Radix::Decimal => format!("{}", i),
                    Radix::Hexadecimal => format!("{:#x}", i),
                },
            )?,
            Literal::Float(string, type_) => typed_number(type_, string.clone())?,
        })
    }
}