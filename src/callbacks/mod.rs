//! Provides rustc callback implementations, which specify how
//! each of the two compilations done by DATIR function.
//!
//! DATIR relies on two compilation passes:
//! 1. A "Gather" pass, which collects information from the HIR/MIR.
//! 2. An "Instrument" pass, which transforms the AST, using the gathered information
//!    to insert appropriate instrumentation.

pub mod gather_orig;
pub mod transform_ast;
