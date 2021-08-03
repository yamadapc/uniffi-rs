/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use anyhow::Result;
use askama::Template;
use heck::{CamelCase, MixedCase, ShoutySnakeCase};
use serde::{Deserialize, Serialize};

use crate::bindings::backend::MemberDeclaration;
use crate::interface::*;
use crate::MergeWith;

use crate::bindings::backend::{ CodeType, TypeIdentifier, LanguageOracle };

mod enum_;
mod string_;
mod fallback;
mod legacy_kt;

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
    oracle: KotlinLanguageOracle,
}
impl<'a> KotlinWrapper<'a> {
    pub fn new(config: Config, ci: &'a ComponentInterface) -> Self {
        // I _think_ I want to make this the holder of the LanguageOracle, which holds the CI.
        // I haven't quite worked out if each type's CodeType should be one type (with defintion_code and lift/lower calling)
        // or two (e.g. enums_::CodeType, enums_::CodeDeclType). a CodeDecl type would be an ideal place to put an askama Template definition,
        //
        Self { config, ci, oracle: Default::default() }
    }

    pub fn members(&self) -> Vec<Box<dyn MemberDeclaration + 'a>> {
        Vec::new().into_iter().chain(
            self.ci
                .iter_enum_definitions()
                .into_iter()
                .map(|e| Box::new(enum_::KotlinEnum::new(e)) as Box<dyn MemberDeclaration>)
        ).collect()
    }
}

#[derive(Default)]
pub struct KotlinLanguageOracle;

impl KotlinLanguageOracle {
    fn create_code_type(&self, type_: TypeIdentifier) -> Box<dyn CodeType> {
        // I really want access to the ComponentInterface here so I can look up the interface::{Enum, Record, Error, Object, etc}
        // However, there's some violence and gore I need to do to (temporarily) make the oracle usable from filters.

        // Some refactor of the templates is needed to make progress here: I think most of the filter functions need to take an &dyn LanguageOracle
        match type_ {
            Type::String => Box::new(string_::StringCodeType),
            Type::Enum(id) => Box::new(enum_::EnumCodeType::new(id)),
            _ => Box::new(fallback::FallbackCodeType::new(type_)),
        }
    }
}

impl LanguageOracle for KotlinLanguageOracle {
    fn find(&self, type_: &TypeIdentifier) -> Result<Box<dyn CodeType>, askama::Error> {
        Ok(
            self.create_code_type(type_.clone())
        )
    }

    /// Get the idiomatic Kotlin rendering of a class name (for enums, records, errors, etc).
    fn class_name(&self, nm: &dyn fmt::Display) -> String {
        nm.to_string().to_camel_case()
    }

    /// Get the idiomatic Kotlin rendering of a function name.
    fn fn_name(&self, nm: &dyn fmt::Display) -> String {
        nm.to_string().to_mixed_case()
    }

    /// Get the idiomatic Kotlin rendering of a variable name.
    fn var_name(&self, nm: &dyn fmt::Display) -> String {
        nm.to_string().to_mixed_case()
    }

    /// Get the idiomatic Kotlin rendering of an individual enum variant.
    fn enum_variant(&self, nm: &dyn fmt::Display) -> String {
        nm.to_string().to_shouty_snake_case()
    }

    /// Get the idiomatic Kotlin rendering of an exception name
    ///
    /// This replaces "Error" at the end of the name with "Exception".  Rust code typically uses
    /// "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
    /// and is distinguished from an "Exception".
    fn exception_name(&self, nm: &dyn fmt::Display) -> String {
        let name = nm.to_string();
        match name.strip_suffix("Error") {
            None => name,
            Some(stripped) => {
                let mut kt_exc_name = stripped.to_owned();
                kt_exc_name.push_str("Exception");
                kt_exc_name
            }
        }
    }

    fn ffi_type_label(&self, ffi_type: &FFIType) -> String {
        match ffi_type {
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
        }
    }
}

pub mod filters {
    use super::*;
    use std::fmt;

    fn oracle() -> impl LanguageOracle {
        KotlinLanguageOracle
    }

    pub fn helper_code(type_: &Type) -> Result<Option<String>, askama::Error> {
        let oracle = oracle();
        Ok(oracle.find(type_)?.helper_code(&oracle))
    }

    pub fn type_kt(type_: &Type) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(oracle.find(type_)?.type_label(&oracle))
    }

    pub fn lower_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(oracle.find(type_)?.lower(&oracle, nm))
    }

    pub fn write_kt(
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
        type_: &Type,
    ) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(oracle.find(type_)?.write(&oracle, nm, target))
    }

    pub fn lift_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(oracle.find(type_)?.lift(&oracle, nm))
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
        Ok(oracle.find(type_)?.literal(&oracle, literal))
    }

    pub fn read_kt(nm: &dyn fmt::Display, type_: &Type) -> Result<String, askama::Error> {
        let oracle = oracle();
        Ok(oracle.find(type_)?.read(&oracle, nm))
    }

    /// Get the Kotlin syntax for representing a given low-level `FFIType`.
    pub fn type_ffi(type_: &FFIType) -> Result<String, askama::Error> {
        Ok(oracle().ffi_type_label(type_))
    }

    /// Get the idiomatic Kotlin rendering of a class name (for enums, records, errors, etc).
    pub fn class_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().class_name(nm))
    }

    /// Get the idiomatic Kotlin rendering of a function name.
    pub fn fn_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().fn_name(nm))
    }

    /// Get the idiomatic Kotlin rendering of a variable name.
    pub fn var_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().var_name(nm))
    }

    /// Get the idiomatic Kotlin rendering of an individual enum variant.
    pub fn enum_variant_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().enum_variant(nm))
    }

    /// Get the idiomatic Kotlin rendering of an exception name
    ///
    /// This replaces "Error" at the end of the name with "Exception".  Rust code typically uses
    /// "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
    /// and is distinguished from an "Exception".
    pub fn exception_name_kt(nm: &dyn fmt::Display) -> Result<String, askama::Error> {
        Ok(oracle().exception_name(nm))
    }
}