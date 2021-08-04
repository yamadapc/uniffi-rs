/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::{bindings::backend::{CodeType, LanguageOracle, Literal, StringReturn, TypeIdentifier}, interface::{Radix, types::Type}};
use paste::paste;
use std::fmt;

fn render_literal(_oracle: &dyn LanguageOracle, literal: &Literal) -> String {
    fn typed_number(type_: &Type, num_str: String) -> String {
        match type_ {
            // Bytes, Shorts and Ints can all be inferred from the type.
            Type::Int8 | Type::Int16 | Type::Int32 => num_str,
            Type::Int64 => format!("{}L", num_str),

            Type::UInt8 | Type::UInt16 | Type::UInt32 => format!("{}u", num_str),
            Type::UInt64 => format!("{}uL", num_str),

            Type::Float32 => format!("{}f", num_str),
            Type::Float64 => num_str,
            _ => panic!("Unexpected literal: {} is not a number", num_str),
        }
    }

    match literal {
        Literal::Boolean(v) => format!("{}", v),
        Literal::String(s) => format!("\"{}\"", s),
        Literal::Int(i, radix, type_) => typed_number(
            &type_,
            match radix {
                Radix::Octal => format!("{:#x}", i),
                Radix::Decimal => format!("{}", i),
                Radix::Hexadecimal => format!("{:#x}", i),
            },
        ),
        Literal::UInt(i, radix, type_) => typed_number(
            &type_,
            match radix {
                Radix::Octal => format!("{:#x}", i),
                Radix::Decimal => format!("{}", i),
                Radix::Hexadecimal => format!("{:#x}", i),
            },
        ),
        Literal::Float(string, type_) => typed_number(&type_, string.clone()),

        _ => unreachable!("Literal"),
    }

}

macro_rules! impl_code_type_for_primitive {
    ($T:ty, $class_name:literal, $helper_string:literal) => {
        paste! {
            pub struct $T;

            impl CodeType for $T  {
                fn type_label(&self, _oracle: &dyn LanguageOracle) -> StringReturn {
                    "Boolean".into()
                }

                fn canonical_name(&self, oracle: &dyn LanguageOracle) -> StringReturn {
                    self.type_label(oracle)
                }

                fn literal(&self, oracle: &dyn LanguageOracle, literal: &Literal) -> StringReturn {
                    render_literal(oracle, &literal)
                }

                fn lower(&self, oracle: &dyn LanguageOracle, nm: &dyn fmt::Display) -> StringReturn {
                    format!("{}.lower({})", $class_name, oracle.var_name(nm))
                }

                fn write(&self, oracle: &dyn LanguageOracle, nm: &dyn fmt::Display, target: &dyn fmt::Display) -> StringReturn {
                    format!("{}.write({}, {})", $class_name, oracle.var_name(nm), target)
                }

                fn lift(&self, _oracle: &dyn LanguageOracle, nm: &dyn fmt::Display) -> StringReturn {
                    format!("{}.lift({})", $class_name, nm)
                }

                fn read(&self, _oracle: &dyn LanguageOracle, nm: &dyn fmt::Display) -> StringReturn {
                    format!("{}.read({})", $class_name, nm)
                }

                fn helper_code(&self, _oracle: &dyn LanguageOracle) -> Option<String> {
                    Some($helper_string.to_string())
                }
            }
        }
    }
}

impl_code_type_for_primitive!(
    BoolCodeType,
    "BoolInternals",
    r#"
internal object BoolInternals {

    fun lift(v: Byte): Boolean {
        return v.toInt() != 0
    }

    fun read(buf: ByteBuffer): Boolean {
        return BoolInternals.lift(buf.get())
    }

    fun lower(b: Boolean): Byte {
        return if (b) 1.toByte() else 0.toByte()
    }

    fun write(b: Boolean, buf: RustBufferBuilder) {
        buf.putByte(BoolInternals.lower(b))
    }
}
"#
);
