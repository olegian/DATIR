//! This module contains all logic used during the second "Instrument" compilation, by
//! [crate::callbacks::transform_ast]. Namely, view [`instrument::InstrumentingVisitor`] for more
//! information.
mod expr;
mod hoisting;
pub mod instrument;
mod item;
mod types;
