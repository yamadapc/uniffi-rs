/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use askama::Error;

use crate::bindings::backend::{CodeType, Literal, TypeOracle};

pub struct EnumCodeType {
    id: String,
}

impl EnumCodeType {
    pub fn new(id: String) -> Self { Self { id } }
}

impl CodeType for EnumCodeType {
    fn type_label(&self, oracle: &dyn TypeOracle) -> Result<String, Error> {
        oracle.class_name(&self.id)
    }

    fn canonical_name(&self, oracle: &dyn TypeOracle) -> Result<String, askama::Error> {
        Ok(format!("Enum{}", self.type_label(oracle)?))
    }

    fn literal(&self, oracle: &dyn TypeOracle, literal: &Literal) -> Result<String, Error> {
        if let Literal::Enum(v, _) = literal {
            Ok(
                format!("{}.{}", self.type_label(oracle)?, oracle.enum_variant(v)?)
            )
        } else {
            unreachable!();
        }
    }

    fn lower(&self, oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> Result<String, Error> {
        Ok(format!("{}.lower()", oracle.var_name(nm)?))
    }

    fn write(&self, oracle: &dyn TypeOracle, nm: &dyn fmt::Display, target: &dyn fmt::Display) -> Result<String, Error> {
        Ok(format!("{}.write({})", oracle.var_name(nm)?, target))
    }

    fn lift(&self, oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> Result<String, Error> {
        Ok(format!("{}.lift({})", self.type_label(oracle)?, nm))
    }

    fn read(&self, oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> Result<String, Error> {
        Ok(format!("{}.read({})", self.type_label(oracle)?, nm))
    }
}