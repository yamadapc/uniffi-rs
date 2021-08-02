/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

 use std::fmt;

 use crate::bindings::backend::{CodeType, Literal, TypeIdentifier, TypeOracle, StringReturn};
 use super::legacy_kt;

pub struct FallbackCodeType {
    type_: TypeIdentifier,
}

impl FallbackCodeType {
    pub fn new(type_: TypeIdentifier) -> Self { Self { type_ } }
}

impl CodeType for FallbackCodeType {
    fn type_label(&self, _oracle: &dyn TypeOracle) -> StringReturn {
        legacy_kt::type_kt(&self.type_).unwrap()
    }

    fn canonical_name(&self, _oracle: &dyn TypeOracle) -> StringReturn {
        self.type_.canonical_name()
    }

    fn literal(&self, _oracle: &dyn TypeOracle, literal: &Literal) -> StringReturn {
        legacy_kt::literal_kt(literal).unwrap()
    }

    fn lower(&self, _oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> StringReturn {
        legacy_kt::lower_kt(nm, &self.type_).unwrap()
    }

    fn write(&self,
        _oracle: &dyn TypeOracle,
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
    ) -> StringReturn {
        legacy_kt::write_kt(nm, target, &self.type_).unwrap()
    }

    fn lift(&self, _oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> StringReturn {
        legacy_kt::lift_kt(nm, &self.type_).unwrap()
    }

    fn read(&self, _oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> StringReturn {
        legacy_kt::read_kt(nm, &self.type_).unwrap()
    }
}
