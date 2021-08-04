/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use crate::bindings::backend::{
    CodeType, LanguageOracle, Literal, MemberDeclaration, StringReturn, TypeIdentifier,
};
use crate::interface::{ComponentInterface, Enum};
use askama::Template;

use super::filters;
pub struct EnumCodeType {
    id: String,
}

impl EnumCodeType {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl CodeType for EnumCodeType {
    fn type_label(&self, oracle: &dyn LanguageOracle) -> StringReturn {
        oracle.class_name(&self.id)
    }

    fn canonical_name(&self, oracle: &dyn LanguageOracle) -> StringReturn {
        format!("Enum{}", self.type_label(oracle))
    }

    fn literal(&self, oracle: &dyn LanguageOracle, literal: &Literal) -> StringReturn {
        if let Literal::Enum(v, _) = literal {
            format!("{}.{}", self.type_label(oracle), oracle.enum_variant(v))
        } else {
            unreachable!();
        }
    }

    fn lower(&self, oracle: &dyn LanguageOracle, nm: &dyn fmt::Display) -> StringReturn {
        format!("{}.lower()", oracle.var_name(nm))
    }

    fn write(
        &self,
        oracle: &dyn LanguageOracle,
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
    ) -> StringReturn {
        format!("{}.write({})", oracle.var_name(nm), target)
    }

    fn lift(&self, oracle: &dyn LanguageOracle, nm: &dyn fmt::Display) -> StringReturn {
        format!("{}.lift({})", self.type_label(oracle), nm)
    }

    fn read(&self, oracle: &dyn LanguageOracle, nm: &dyn fmt::Display) -> StringReturn {
        format!("{}.read({})", self.type_label(oracle), nm)
    }

    fn helper_code(&self, oracle: &dyn LanguageOracle) -> Option<String> {
        Some(format!(
            "// Helper code for {} enum is found in EnumTemplate.kt",
            self.type_label(oracle)
        ))
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "EnumTemplate.kt")]
pub struct KotlinEnum {
    inner: Enum,
    contains_unsigned_types: bool,
    contains_object_references: bool,
}

impl KotlinEnum {
    pub fn new(inner: Enum, ci: &ComponentInterface) -> Self {
        Self {
            contains_unsigned_types: inner.contains_unsigned_types(ci),
            contains_object_references: inner.contains_object_references(ci),
            inner,
        }
    }
    pub fn inner(&self) -> &Enum {
        &self.inner
    }
    pub fn contains_object_references(&self) -> bool {
        self.contains_object_references
    }
    pub fn contains_unsigned_types(&self) -> bool {
        self.contains_unsigned_types
    }
}

impl MemberDeclaration for KotlinEnum {
    fn type_identifier(&self) -> TypeIdentifier {
        TypeIdentifier::Enum(self.inner.name().into())
    }

    fn definition_code(&self, _oracle: &dyn LanguageOracle) -> Option<String> {
        Some(self.render().unwrap())
    }
}
