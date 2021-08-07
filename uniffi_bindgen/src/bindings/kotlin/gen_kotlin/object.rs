/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use crate::bindings::backend::{
    CodeType, LanguageOracle, Literal, MemberDeclaration, TypeIdentifier,
};
use crate::interface::{ComponentInterface, Object};
use askama::Template;

use super::filters;
pub struct ObjectCodeType {
    id: String,
}

impl ObjectCodeType {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl CodeType for ObjectCodeType {
    fn type_label(&self, oracle: &dyn LanguageOracle) -> String {
        oracle.class_name(&self.id)
    }

    fn canonical_name(&self, oracle: &dyn LanguageOracle) -> String {
        format!("Object{}", self.type_label(oracle))
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
            "// Helper code for {} class is found in ObjectTemplate.kt",
            self.type_label(oracle)
        ))
    }

    fn import_code(&self, _oracle: &dyn LanguageOracle) -> Option<Vec<String>> {
        Some(
            vec![
                "java.util.concurrent.atomic.AtomicLong",
                "java.util.concurrent.atomic.AtomicBoolean",
            ]
            .into_iter()
            .map(|s| s.into())
            .collect(),
        )
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "ObjectTemplate.kt")]
pub struct KotlinObject {
    inner: Object,
    contains_unsigned_types: bool,
}

impl KotlinObject {
    pub fn new(inner: Object, ci: &ComponentInterface) -> Self {
        Self {
            contains_unsigned_types: inner.contains_unsigned_types(ci),
            inner,
        }
    }
    pub fn inner(&self) -> &Object {
        &self.inner
    }
    pub fn contains_unsigned_types(&self) -> bool {
        self.contains_unsigned_types
    }
}

impl MemberDeclaration for KotlinObject {
    fn type_identifier(&self) -> TypeIdentifier {
        TypeIdentifier::Object(self.inner.name().into())
    }

    fn definition_code(&self, _oracle: &dyn LanguageOracle) -> Option<String> {
        Some(self.render().unwrap())
    }
}

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "ObjectRuntime.kt")]
pub struct KotlinObjectRuntime {
    is_needed: bool,
}

impl KotlinObjectRuntime {
    pub fn new(ci: &ComponentInterface) -> Self {
        Self {
            is_needed: !ci.iter_object_definitions().is_empty(),
        }
    }
}

impl MemberDeclaration for KotlinObjectRuntime {
    fn type_identifier(&self) -> TypeIdentifier {
        unreachable!("Runtime code is not a member")
    }

    fn definition_code(&self, _oracle: &dyn LanguageOracle) -> Option<String> {
        if self.is_needed {
            Some(self.render().unwrap())
        } else {
            None
        }
    }
}
