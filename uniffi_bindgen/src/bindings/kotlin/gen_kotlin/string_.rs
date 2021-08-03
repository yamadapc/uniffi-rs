/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

 use std::fmt;

 use crate::bindings::backend::{CodeType, Literal, LanguageOracle, StringReturn};

 pub struct StringCodeType;

 impl CodeType for StringCodeType {
     fn type_label(&self, _oracle: &dyn LanguageOracle) -> StringReturn {
         "String".into()
     }

     fn canonical_name(&self, oracle: &dyn LanguageOracle) -> StringReturn {
         self.type_label(oracle)
     }

     fn literal(&self, _oracle: &dyn LanguageOracle, literal: &Literal) -> StringReturn {
         if let Literal::String(v) = literal {
             format!("\"{}\"", v)
         } else {
             unreachable!();
         }
     }

     fn lower(&self, oracle: &dyn LanguageOracle, nm: &dyn fmt::Display) -> StringReturn {
         format!("StringInternals.lower({})", oracle.var_name(nm))
     }

     fn write(&self, oracle: &dyn LanguageOracle, nm: &dyn fmt::Display, target: &dyn fmt::Display) -> StringReturn {
         format!("StringInternals.write({}, {})", oracle.var_name(nm), target)
     }

     fn lift(&self, _oracle: &dyn LanguageOracle, nm: &dyn fmt::Display) -> StringReturn {
         format!("StringInternals.lift({})", nm)
     }

     fn read(&self, _oracle: &dyn LanguageOracle, nm: &dyn fmt::Display) -> StringReturn {
         format!("StringInternals.read({})", nm)
     }

     fn helper_code(&self, _oracle: &dyn LanguageOracle) -> Option<String> {
         Some(HELPER_CODE.clone())
     }
 }

 lazy_static::lazy_static! {
    static ref HELPER_CODE: String = r#"
    internal class StringInternals {
        companion object {
            fun lower(s: String): RustBuffer.ByValue {
                val byteArr = s.toByteArray(Charsets.UTF_8)
                // Ideally we'd pass these bytes to `ffi_bytebuffer_from_bytes`, but doing so would require us
                // to copy them into a JNA `Memory`. So we might as well directly copy them into a `RustBuffer`.
                val rbuf = RustBuffer.alloc(byteArr.size)
                rbuf.asByteBuffer()!!.put(byteArr)
                return rbuf
            }

            fun write(s: String, buf: RustBufferBuilder) {
                val byteArr = s.toByteArray(Charsets.UTF_8)
                buf.putInt(byteArr.size)
                buf.put(byteArr)
            }

            fun lift(rbuf: RustBuffer.ByValue): String {
                try {
                    val byteArr = ByteArray(rbuf.len)
                    rbuf.asByteBuffer()!!.get(byteArr)
                    return byteArr.toString(Charsets.UTF_8)
                } finally {
                    RustBuffer.free(rbuf)
                }
            }

            fun read(buf: ByteBuffer): String {
                val len = buf.getInt()
                val byteArr = ByteArray(len)
                buf.get(byteArr)
                return byteArr.toString(Charsets.UTF_8)
            }
        }
    }
    "#.to_string();
 }