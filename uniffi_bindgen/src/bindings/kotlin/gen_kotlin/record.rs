/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use crate::bindings::backend::{
    CodeType, LanguageOracle, Literal, MemberDeclaration, StringReturn, TypeIdentifier,
};
use crate::interface::{ComponentInterface, Record};
use askama::Template;

use super::filters;
pub struct RecordCodeType {
    id: String,
}

impl RecordCodeType {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl CodeType for RecordCodeType {
    fn type_label(&self, oracle: &dyn LanguageOracle) -> StringReturn {
        oracle.class_name(&self.id)
    }

    fn canonical_name(&self, oracle: &dyn LanguageOracle) -> StringReturn {
        format!("Record{}", self.type_label(oracle))
    }

    fn literal(&self, _oracle: &dyn LanguageOracle, _literal: &Literal) -> StringReturn {
        unreachable!();
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
            "// Helper code for {} record is found in RecordTemplate.kt",
            self.type_label(oracle)
        ))
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "RecordTemplate.kt")]
pub struct KotlinRecord {
    inner: Record,
    contains_unsigned_types: bool,
    contains_object_references: bool,
}

impl KotlinRecord {
    pub fn new(inner: Record, ci: &ComponentInterface) -> Self {
        Self {
            contains_unsigned_types: inner.contains_unsigned_types(ci),
            contains_object_references: inner.contains_object_references(ci),
            inner,
        }
    }
    pub fn inner(&self) -> &Record {
        &self.inner
    }
    pub fn contains_object_references(&self) -> bool {
        self.contains_object_references
    }
    pub fn contains_unsigned_types(&self) -> bool {
        self.contains_unsigned_types
    }
}

impl MemberDeclaration for KotlinRecord {
    fn type_identifier(&self) -> TypeIdentifier {
        TypeIdentifier::Record(self.inner.name().into())
    }

    fn definition_code(&self, _oracle: &dyn LanguageOracle) -> Option<String> {
        Some(self.render().unwrap())
    }
}
