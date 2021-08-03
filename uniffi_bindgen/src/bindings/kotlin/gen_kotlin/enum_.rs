/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use crate::{bindings::backend::{CodeType, Literal, LanguageOracle, StringReturn}, interface::Enum};

pub struct EnumCodeType {
    id: String,
}

impl EnumCodeType {
    pub fn new(id: String) -> Self { Self { id } }
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

    fn write(&self, oracle: &dyn LanguageOracle, nm: &dyn fmt::Display, target: &dyn fmt::Display) -> StringReturn {
        format!("{}.write({})", oracle.var_name(nm), target)
    }

    fn lift(&self, oracle: &dyn LanguageOracle, nm: &dyn fmt::Display) -> StringReturn {
        format!("{}.lift({})", self.type_label(oracle), nm)
    }

    fn read(&self, oracle: &dyn LanguageOracle, nm: &dyn fmt::Display) -> StringReturn {
        format!("{}.read({})", self.type_label(oracle), nm)
    }

    fn helper_code(&self, oracle: &dyn LanguageOracle) -> Option<String> {
        Some(format!("// {} Arrived!", self.type_label(oracle)))
    }
}

struct KotlinEnum<'enum_> {
    pub inner: &'enum_ Enum
}