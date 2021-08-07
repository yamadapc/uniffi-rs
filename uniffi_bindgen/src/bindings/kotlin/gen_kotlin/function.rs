/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::bindings::backend::{LanguageOracle, MemberDeclaration, TypeIdentifier};
use crate::interface::{ComponentInterface, Function};
use askama::Template;

use super::filters;

#[derive(Template)]
#[template(syntax = "kt", escape = "none", path = "TopLevelFunctionTemplate.kt")]
pub struct KotlinFunction {
    inner: Function,
    contains_unsigned_types: bool,
}

impl KotlinFunction {
    pub fn new(inner: Function, ci: &ComponentInterface) -> Self {
        Self {
            contains_unsigned_types: inner.contains_unsigned_types(ci),
            inner,
        }
    }
    pub fn inner(&self) -> &Function {
        &self.inner
    }
    pub fn contains_unsigned_types(&self) -> bool {
        self.contains_unsigned_types
    }
}

impl MemberDeclaration for KotlinFunction {
    fn type_identifier(&self) -> TypeIdentifier {
        // XXX I'm not convinced this method should be part of MemberDeclaration.
        unreachable!("Functions should not be passed around")
    }

    fn definition_code(&self, _oracle: &dyn LanguageOracle) -> Option<String> {
        Some(self.render().unwrap())
    }
}
