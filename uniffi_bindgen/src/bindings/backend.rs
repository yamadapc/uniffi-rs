/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::interface::*;
use std::fmt;

pub type TypeIdentifier = Type;
pub type Literal = crate::interface::Literal;

// Placeholder return type, while we do refactoring.
pub type StringReturn = String;

pub trait TypeOracle {
    fn find(&self, type_: &TypeIdentifier) -> Result<Box<dyn CodeType>, askama::Error>;

    /// Get the idiomatic Kotlin rendering of a class name (for enums, records, errors, etc).
    fn class_name(&self, nm: &dyn fmt::Display) -> String;

    /// Get the idiomatic Kotlin rendering of a function name.
    fn fn_name(&self, nm: &dyn fmt::Display) -> String;

    /// Get the idiomatic Kotlin rendering of a variable name.
    fn var_name(&self, nm: &dyn fmt::Display) -> String;

    /// Get the idiomatic Kotlin rendering of an individual enum variant.
    fn enum_variant(&self, nm: &dyn fmt::Display) -> String;

    /// Get the idiomatic Kotlin rendering of an exception name
    ///
    /// This replaces "Error" at the end of the name with "Exception".  Rust code typically uses
    /// "Error" for any type of error but in the Java world, "Error" means a non-recoverable error
    /// and is distinguished from an "Exception".
    fn exception_name(&self, nm: &dyn fmt::Display) -> String;
}

pub trait CodeType {
    fn type_label(&self, oracle: &dyn TypeOracle) -> StringReturn;

    fn canonical_name(&self, oracle: &dyn TypeOracle) -> StringReturn;

    fn literal(&self, oracle: &dyn TypeOracle, literal: &Literal) -> StringReturn;
    /// Get a Kotlin expression for lowering a value into something we can pass over the FFI.
    ///
    /// Where possible, this delegates to a `lower()` method on the type itself, but special
    /// handling is required for some compound data types.
    fn lower(&self, oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> StringReturn;

    /// Get a Kotlin expression for writing a value into a byte buffer.
    ///
    /// Where possible, this delegates to a `write()` method on the type itself, but special
    /// handling is required for some compound data types.
    fn write(&self, oracle: &dyn TypeOracle, nm: &dyn fmt::Display, target: &dyn fmt::Display) -> StringReturn;

    /// Get a Kotlin expression for lifting a value from something we received over the FFI.
    ///
    /// Where possible, this delegates to a `lift()` method on the type itself, but special
    /// handling is required for some compound data types.
    fn lift(&self, oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> StringReturn;

    /// Get a Kotlin expression for reading a value from a byte buffer.
    ///
    /// Where possible, this delegates to a `read()` method on the type itself, but special
    /// handling is required for some compound data types.
    fn read(&self, oracle: &dyn TypeOracle, nm: &dyn fmt::Display) -> StringReturn;
}
