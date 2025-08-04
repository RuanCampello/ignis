//! This module is responsible for parsing and representing `.class` files as defined by the Java Virtual Machine specification.
//!
//! Which include things like:
//! - Low-level binary parsing of `.class` files, including constant pool, fields, methods, and attributes.
//! - Data structures to represent class file components in memory.
//! - Validation of class file format and version.
//!
//! The output of this module is a structured `ClassFile` representation, which is used by the class loader and interpreter.

mod constant_pool;
