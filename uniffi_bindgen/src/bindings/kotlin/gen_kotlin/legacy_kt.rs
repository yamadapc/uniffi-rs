/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use super::*;
use std::fmt;

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
        Type::Optional(_) | Type::Sequence(_) | Type::Map(_) | Type::Timestamp | Type::Duration => {
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
        Type::Optional(_) | Type::Sequence(_) | Type::Map(_) | Type::Timestamp | Type::Duration => {
            format!(
                "write{}({}, {})",
                class_name_kt(&type_.canonical_name())?,
                nm,
                target,
            )
        }
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
        Type::Optional(_) | Type::Sequence(_) | Type::Map(_) | Type::Timestamp | Type::Duration => {
            format!("lift{}({})", class_name_kt(&type_.canonical_name())?, nm)
        }
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
        Type::Optional(_) | Type::Sequence(_) | Type::Map(_) | Type::Timestamp | Type::Duration => {
            format!("read{}({})", class_name_kt(&type_.canonical_name())?, nm)
        }
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
