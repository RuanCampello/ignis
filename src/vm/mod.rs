//! This module implements the core of the Java Virtual Machine, providing the runtime environment necessary to execute Java bytecode.
//! It handles the execution of instructions defined by the JVM specification, managing the stack frames, operand stacks,
//! and local variables for each method invocation.
//!
//! The module is responsible for maintaining the lifecycle of objects, handling method calls and returns, and supporting control flow operations.
//! It also manages the interaction with the runtime constant pool and resolves symbolic references during execution.
//! This module acts as the bridge between the static class file data and the dynamic execution of Java programs,
//! forming the heart of the JVM interpreter and class loader runtime system.

mod instructions;
