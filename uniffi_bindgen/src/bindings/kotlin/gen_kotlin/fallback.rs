/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

 use std::fmt;

 use askama::Error;

 use crate::bindings::backend::{CodeType, Literal, TypeIdentifier, TypeOracle};
 use super::legacy_kt;

pub struct FallbackCodeType {
    type_: TypeIdentifier,
}

impl FallbackCodeType {
    pub fn new(type_: TypeIdentifier) -> Self { Self { type_ } }

    fn type_identifier(&self, _oracle: &dyn TypeOracle) -> &TypeIdentifier {
        &self.type_
    }
}

impl CodeType for FallbackCodeType {
    fn type_label(&self, oracle: &dyn TypeOracle) -> Result<String, Error> {
        let type_ = self.type_identifier(oracle);
        legacy_kt::type_kt(type_)
    }

    fn canonical_name(&self, oracle: &dyn TypeOracle) -> Result<String, Error> {
        let type_ = self.type_identifier(oracle);
        Ok(type_.canonical_name())
    }

    fn literal(&self, _oracle: &dyn TypeOracle, literal: &Literal) -> Result<String, Error> {
        legacy_kt::literal_kt(literal)
    }

    fn lower(&self, oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> Result<String, Error> {
        legacy_kt::lower_kt(nm, self.type_identifier(oracle))
    }

    fn write(&self,
        oracle: &dyn TypeOracle,
        nm: &dyn fmt::Display,
        target: &dyn fmt::Display,
    ) -> Result<String, Error> {
        legacy_kt::write_kt(nm, target, self.type_identifier(oracle))
    }

    fn lift(&self, oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> Result<String, Error> {
        legacy_kt::lift_kt(nm, self.type_identifier(oracle))
    }

    fn read(&self, oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> Result<String, Error> {
        legacy_kt::read_kt(nm, self.type_identifier(oracle))
    }
}
