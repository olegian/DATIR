/* Because we are invoking the compiler multiple times, we need some
 * way of relaying information between the multiple compilations. This file
 * defines some structs which can be used for just that.
 *
 * FirstPassInfo is used to relay information from the first pass, which
 * discovers what functions we are going to be instrumenting and where we are
 * making calls to untracked functions.
 *
 * FirstPassInfo is then used to during the second compilation to only
 * instrument specific functions, during which StubInfo is constructed.
 * StubInfo is used to record the updated data types used in function
 * inputs and outputs, as well as the function name and parameter names.
 * StubInfo is then consumed by the stub creation process, to add in
 * the correct stubs responsible for managing sites.
*/

use std::collections::{HashMap, HashSet};

use rustc_hir::def_id::DefId;
use rustc_middle as mir;
use rustc_span::{Ident, Span};

use crate::common::CanBeTupled;

/// Information about a field in an external struct that needs a tagged variant
#[derive(Debug, Clone)]
pub struct StructFieldInfo {
    /// Name of the field
    pub name: String,
    /// String representation of the original type (e.g., "u32")
    pub ty_str: String,
    /// Whether this field should be wrapped in Tagged<T>
    pub should_tuple: bool,
}

/// Information about an external struct type that needs a tagged variant
#[derive(Debug, Clone)]
pub struct CompoundTypeInfo {
    /// Name of the original type (e.g., "Foo")
    pub original_name: String,
    /// Name for the tagged variant (e.g., "__ati_Foo")
    pub tagged_name: String,
    /// Fields with their tagging info
    pub fields: Vec<StructFieldInfo>,
}

/// What kind of return value an untracked function call produces
#[derive(Debug, Clone)]
pub enum UntrackedReturnKind {
    /// Return type is a tupleable primitive — wrap in ATI::track()
    Tupleable,
    /// Return type is an external struct — convert to tagged variant
    Compound(String), // tagged_name, key into compound_types map
    /// Return type doesn't need special handling (unit, unsupported, etc.)
    None,
}

/// Contains all information that is going to be passed between the
/// first and second compilation rounds. Populated by invoking the
/// compiler, using the GatherAtiInfo callbacks.
#[derive(Debug)]
pub struct FirstPassInfo {
    /// which user-defined functions are instrumented across the entire project
    tracked_fn_def_ids: HashSet<DefId>,
    tracked_fn_idents: HashSet<Ident>,

    /// places where a track_slice needs to be inserted, as a coercion from an array to a slice type occurred
    array_to_slice_locs: HashSet<Span>,
    index_by_range_locs: HashSet<Span>,

    /// which user-defined types (structs, enums) are tracked
    tracked_type_idents: HashSet<String>,

    /// places where a non-tracked function is called, mapped to info about the return type
    untracked_fn_calls: HashMap<Span, UntrackedReturnKind>,

    /// places where indexing into an untracked container type occurs, mapped to return kind
    untracked_index_locs: HashMap<Span, UntrackedReturnKind>,

    /// external struct types that need tagged variant generation, keyed by tagged_name
    compound_types: HashMap<String, CompoundTypeInfo>,
}

impl Default for FirstPassInfo {
    fn default() -> Self {
        Self {
            tracked_fn_def_ids: Default::default(),
            tracked_fn_idents: Default::default(),
            tracked_type_idents: Default::default(),
            untracked_fn_calls: Default::default(),
            untracked_index_locs: Default::default(),
            compound_types: Default::default(),

            array_to_slice_locs: Default::default(),
            index_by_range_locs: Default::default(),
        }
    }
}

impl FirstPassInfo {
    /// register that a function with `ident` and `def_id` should
    /// instrumented later
    // NOTE: This is only really useful for extern crates and library files
    // that we are unable to instrument. For now, there is no reason to do this
    // as we assume that all code
    pub fn observe_tracked_fn(&mut self, ident: &Ident, def_id: DefId) {
        self.tracked_fn_idents.insert(ident.clone());
        self.tracked_fn_def_ids.insert(def_id);
    }

    /// register that a type with `name` is user-defined and tracked
    pub fn observe_tracked_type(&mut self, name: String) {
        self.tracked_type_idents.insert(name);
    }

    /// register that a function call was made to an untracked function at
    /// `loc`, with the given return kind
    pub fn observe_untracked_fn_call(&mut self, loc: Span, kind: UntrackedReturnKind) {
        self.untracked_fn_calls.insert(loc, kind);
    }

    /// register that an external struct type needs a tagged variant
    pub fn register_compound_type(&mut self, info: CompoundTypeInfo) {
        self.compound_types
            .entry(info.tagged_name.clone())
            .or_insert(info);
    }

    /// register that at this `loc`, an array was implicitly coereced to a slice
    /// (which requires going from a Tagged<[T; N]> to a Tagged<&[T]>)
    pub fn observe_slice_coercion(&mut self, loc: Span) {
        self.array_to_slice_locs.insert(loc);
    }

    /// register that an index expression at `loc` targets an untracked container type
    pub fn observe_untracked_index(&mut self, loc: Span, kind: UntrackedReturnKind) {
        self.untracked_index_locs.insert(loc, kind);
    }

    pub fn observe_index_by_range(&mut self, loc: Span) {
        self.index_by_range_locs.insert(loc);
    }

    /// returns true if this type name represents a user-defined (tracked) type
    pub fn is_type_tracked(&self, name: &str) -> bool {
        self.tracked_type_idents.contains(name)
    }

    /// returns true if this identifier represent a tracked function
    pub fn is_fn_ident_tracked(&self, ident: &Ident) -> bool {
        self.tracked_fn_idents.contains(ident)
    }

    /// returns true if this def_id represents a tracked function
    pub fn is_fn_def_id_tracked(&self, def_id: &DefId) -> bool {
        self.tracked_fn_def_ids.contains(def_id)
    }

    /// returns the return kind of an untracked function call at this location,
    /// or None if no untracked call was recorded here
    pub fn get_untracked_return_kind(&self, location: &Span) -> Option<&UntrackedReturnKind> {
        self.untracked_fn_calls.get(location)
    }

    /// returns all compound types that need tagged variant generation
    pub fn compound_types(&self) -> &HashMap<String, CompoundTypeInfo> {
        &self.compound_types
    }

    /// If a type name matches an external struct with a tagged variant,
    /// return the tagged variant name.
    pub fn get_tagged_name_for_type(&self, type_name: &str) -> Option<&str> {
        self.compound_types
            .values()
            .find(|info| info.original_name == type_name)
            .map(|info| info.tagged_name.as_str())
    }

    /// returns the return kind of an untracked index expression at this location,
    /// or None if no untracked index was recorded here
    pub fn get_untracked_index_kind(&self, location: &Span) -> Option<&UntrackedReturnKind> {
        self.untracked_index_locs.get(location)
    }

    pub fn is_span_ref_type_coercion(&self, location: &Span) -> bool {
        self.array_to_slice_locs.contains(location)
    }

    pub fn is_span_index_by_range(&self, location: &Span) -> bool {
        self.index_by_range_locs.contains(location)
    }
}
