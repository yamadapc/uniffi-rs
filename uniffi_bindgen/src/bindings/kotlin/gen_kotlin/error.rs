/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use crate::bindings::backend::{
    CodeType, LanguageOracle, Literal, MemberDeclaration, TypeIdentifier,
};
use crate::interface::{ComponentInterface, Error};
use askama::Template;

use super::filters;
pub struct ErrorCodeType {
    id: String,
}

impl ErrorCodeType {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl CodeType for ErrorCodeType {
    fn type_label(&self, oracle: &dyn LanguageOracle) -> String {
        oracle.class_name(&self.id)
    }

    fn canonical_name(&self, oracle: &dyn LanguageOracle) -> String {
        format!("Error{}", self.type_label(oracle))
    }

    fn literal(&self, _oracle: &dyn LanguageOracle, _literal: &Literal) -> String {
        unreachable!();
    }

    fn lower(&self, oracle: &dyn LanguageOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.lower()", oracle.var_name(nm))
    }

    fn write(
        &self,
        oracle: &dyn LanguageOracle,
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
    ) -> String {
        format!("{}.write({})", oracle.var_name(nm), target)
    }

    fn lift(&self, oracle: &dyn LanguageOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.lift({})", self.type_label(oracle), nm)
    }

    fn read(&self, oracle: &dyn LanguageOracle, nm: &dyn fmt::Display) -> String {
        format!("{}.read({})", self.type_label(oracle), nm)
    }

    fn helper_code(&self, oracle: &dyn LanguageOracle) -> Option<String> {
        Some(format!(
            "// Helper code for {} enum is found in ErrorTemplate.kt",
            self.type_label(oracle)
        ))
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "ErrorTemplate.kt")]
pub struct KotlinError {
    inner: Error,
    contains_object_references: bool,
}

impl KotlinError {
    pub fn new(inner: Error, ci: &ComponentInterface) -> Self {
        Self {
            contains_object_references: inner.contains_object_references(ci),
            inner,
        }
    }
    pub fn inner(&self) -> &Error {
        &self.inner
    }
    pub fn contains_object_references(&self) -> bool {
        self.contains_object_references
    }
}

impl MemberDeclaration for KotlinError {
    fn type_identifier(&self) -> TypeIdentifier {
        TypeIdentifier::Error(self.inner.name().into())
    }

    fn definition_code(&self, _oracle: &dyn LanguageOracle) -> Option<String> {
        Some(self.render().unwrap())
    }
}
