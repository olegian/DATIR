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
use rustc_span::source_map::SourceMap;
use rustc_span::{Ident, Span};

use crate::{
    common::CanBeTupled,
    gather::{span_key::SpanKey, type_key::TypeKey},
};

/// ::-joined module path matching `tcx.def_path_str` format.
/// `""` denotes the crate root.
pub type ModPath = String;
/// Contains per-fn information learned in pass 1
#[derive(Debug, Clone)]
pub struct FnEntry {
    pub ident: Ident,
    pub def_id: DefId,

    /// DeclsFile::ppt_base_name for this function.
    pub base_ppt_name: String,
}

/// All instrumented fns defined in a single module, partitioned by their
/// namespace (free fn vs. method on a type).
///
/// It's important to use Strings rather than any kind of Symbol (or anything
/// that is compile-session dependant) as this needs to be stable between
/// the two compilation passes
#[derive(Debug, Default)]
pub struct ModEntry {
    /// Free functions in this module, keyed by fn name.
    pub free_fns: HashMap<String, FnEntry>,
    /// Methods, keyed by `TypeKey` and then by method name.
    pub methods: HashMap<TypeKey, HashMap<String, FnEntry>>,
}

/// Contains all information that is going to be passed between the
/// first and second compilation rounds. Populated by invoking the
/// compiler, using the GatherAtiInfo callbacks.
///
/// All span-keyed sets store their entries as `SpanKey`, and every
/// `observe_*` / `is_*` method routes the input `Span` through
/// `SpanKey::from_span`. That helper:
///   * collapses macro-expansion / desugaring spans via
///     `source_callsite()` so a query on the syntactic AST node hits the
///     entry recorded against the underlying user-source span;
///   * rejects dummy / unmapped spans;
///   * normalizes to `(file, lo_byte_offset, hi_byte_offset)` so the
///     keys are immune to span-interning quirks across separate
///     compiler invocations.
/// Pass-1 sites (HIR analyzers) and pass-2 sites (AST visitor) MUST go
/// through these helpers; never insert into / look up the inner sets
/// directly with a raw Span.
#[derive(Debug, Default)]
pub struct FirstPassInfo {
    /// Per-module index of every fn/method that pass 1 wants pass 2 to
    /// instrument. Note that ModEntry will also hold the appropriate
    /// base_ppt_name to use when constructing sites.
    mods: HashMap<ModPath, ModEntry>,

    /// Flat set of every tracked fn/method's DefId. Maintained as a side
    /// cache when entries are added to mods because find_calls.rs needs to
    /// determine if a particular call is tracked only given a DefId resolved by typeck.
    tracked_fn_def_ids: HashSet<DefId>,

    /// places where a track_slice needs to be inserted, as a coercion from an array to a slice type occurred
    index_by_range_locs: HashSet<SpanKey>,

    /// places where a non-tracked function is called
    /// mapped to whether the return type at that call site is tupleable (i.e. a tracked primitive).
    // FIXME: these function calls could return complex types, like structs, which can be tupled but that requires
    // defining a new struct with Tagged variants of all fields, and that's hard to do :(, ignoring for now.
    // hopefully it won't be a problem...
    untracked_fn_calls: HashMap<SpanKey, bool>,

    /// Spans of Ref expressions, which refer to a type T which is tupleable.
    ref_to_tupleable_ty: HashSet<SpanKey>,

    /// Spans of unary `*` expressions whose operand's type is `&T` / `&mut T`
    /// with `T` tupleable. Post-instrumentation these operate on a
    /// `TaggedRef` / `TaggedRefMut`, and a raw `*` would strip the tag.
    tag_stripping_deref_locs: HashSet<SpanKey>,

    /// Spans of `Assign` / `AssignOp` whose LHS is `*expr` with `expr` typed
    /// `&mut T` and `T` tupleable.
    assign_through_tagged_ref_mut_locs: HashSet<SpanKey>,

    /// Spans of expressions whose post-instrumentation type is
    /// `TaggedRefMut<T>` — i.e. their typeck-resolved (post-adjustment) type
    /// is `&mut T` with `T` tupleable. `TaggedRefMut` is move-only, so any
    /// pass-2 rewrite that *consumes* such an expression (binding it into
    /// `let __ati_lhs = ...`, moving it into emitted args) must reborrow
    /// instead, otherwise the original binding is invalidated for any use
    /// later in the function. The reborrow is always semantically safe in
    /// operand position.
    ref_mut_to_tupleable_locs: HashSet<SpanKey>,
}

impl FirstPassInfo {
    /// Record that `def_id` (with display `ident`, item span `item_span`, and
    /// `DeclsFile`-format `base_ppt_name`) lives at `mod_path` and should be
    /// instrumented. `type_key` is `Some(t)` for impl methods on type `t` (head
    /// ident, no generic args) and `None` for free fns
    pub fn observe_fn(
        &mut self,
        mod_path: ModPath,
        type_key: Option<TypeKey>,
        ident: Ident,
        def_id: DefId,
        base_ppt_name: String,
    ) {
        let entry = FnEntry {
            ident,
            def_id,
            base_ppt_name,
        };

        let key = ident.as_str().to_string();
        let mod_entry = self.mods.entry(mod_path).or_default();
        match type_key {
            None => {
                mod_entry.free_fns.insert(key, entry);
            }
            Some(tk) => {
                mod_entry.methods.entry(tk).or_default().insert(key, entry);
            }
        }

        self.tracked_fn_def_ids.insert(def_id);
    }

    /// register that a function call was made to an untracked function at
    /// `loc`, which returned a value of type `ty`
    pub fn observe_untracked_fn_call<'a>(
        &mut self,
        loc: Span,
        sm: &SourceMap,
        ty: mir::ty::Ty<'a>,
    ) {
        if let Some(key) = SpanKey::from_span(loc, sm) {
            self.untracked_fn_calls.insert(key, ty.can_be_tupled());
        }
    }

    pub fn observe_index_by_range(&mut self, loc: Span, sm: &SourceMap) {
        if let Some(key) = SpanKey::from_span(loc, sm) {
            self.index_by_range_locs.insert(key);
        }
    }

    pub fn observe_ref_to_tupleable_ty(&mut self, loc: Span, sm: &SourceMap) {
        if let Some(key) = SpanKey::from_span(loc, sm) {
            self.ref_to_tupleable_ty.insert(key);
        }
    }

    pub fn is_span_ref_to_tupleable_ty(&self, loc: Span, sm: &SourceMap) -> bool {
        SpanKey::from_span(loc, sm)
            .as_ref()
            .map_or(false, |k| self.ref_to_tupleable_ty.contains(k))
    }

    pub fn observe_tag_stripping_deref(&mut self, loc: Span, sm: &SourceMap) {
        if let Some(key) = SpanKey::from_span(loc, sm) {
            self.tag_stripping_deref_locs.insert(key);
        }
    }

    pub fn observe_assign_through_tagged_ref_mut(&mut self, loc: Span, sm: &SourceMap) {
        if let Some(key) = SpanKey::from_span(loc, sm) {
            self.assign_through_tagged_ref_mut_locs.insert(key);
        }
    }

    pub fn is_tag_stripping_deref(&self, loc: Span, sm: &SourceMap) -> bool {
        SpanKey::from_span(loc, sm)
            .as_ref()
            .map_or(false, |k| self.tag_stripping_deref_locs.contains(k))
    }

    pub fn is_assign_through_tagged_ref_mut(&self, loc: Span, sm: &SourceMap) -> bool {
        SpanKey::from_span(loc, sm).as_ref().map_or(false, |k| {
            self.assign_through_tagged_ref_mut_locs.contains(k)
        })
    }

    pub fn observe_ref_mut_to_tupleable(&mut self, loc: Span, sm: &SourceMap) {
        if let Some(key) = SpanKey::from_span(loc, sm) {
            self.ref_mut_to_tupleable_locs.insert(key);
        }
    }

    pub fn is_ref_mut_to_tupleable(&self, loc: Span, sm: &SourceMap) -> bool {
        SpanKey::from_span(loc, sm)
            .as_ref()
            .map_or(false, |k| self.ref_mut_to_tupleable_locs.contains(k))
    }

    /// returns true if this def_id represents a tracked function
    pub fn is_fn_def_id_tracked(&self, def_id: &DefId) -> bool {
        self.tracked_fn_def_ids.contains(def_id)
    }

    /// returns whether the return type of an untracked function call at this
    /// location is tupleable, if such a call exists
    pub fn is_untracked_call_ret_tupleable(&self, location: Span, sm: &SourceMap) -> Option<bool> {
        let key = SpanKey::from_span(location, sm)?;
        self.untracked_fn_calls.get(&key).copied()
    }

    pub fn is_span_index_by_range(&self, location: Span, sm: &SourceMap) -> bool {
        SpanKey::from_span(location, sm)
            .as_ref()
            .map_or(false, |k| self.index_by_range_locs.contains(k))
    }

    /// Look up the recorded `FnEntry` for a free fn in module `mod_path` with
    /// name `ident`
    pub fn lookup_free_fn(&self, mod_path: &str, ident: &str) -> Option<&FnEntry> {
        self.mods.get(mod_path)?.free_fns.get(ident)
    }

    /// Look up the recorded `FnEntry` for a method in module `mod_path` on
    /// the impl identified by `type_key` with name `ident`
    pub fn lookup_method(
        &self,
        mod_path: &str,
        type_key: &TypeKey,
        ident: &str,
    ) -> Option<&FnEntry> {
        self.mods.get(mod_path)?.methods.get(type_key)?.get(ident)
    }

    /// Set of fn/method names defined in a particular `(mod_path, namespace)`
    /// slot, used by stub generation to choose a non-clashing inner name.
    pub fn known_fn_names_in(&self, mod_path: &str, type_key: Option<&TypeKey>) -> HashSet<String> {
        let Some(mod_entry) = self.mods.get(mod_path) else {
            return HashSet::new();
        };
        match type_key {
            None => mod_entry.free_fns.keys().cloned().collect(),
            Some(tk) => mod_entry
                .methods
                .get(tk)
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default(),
        }
    }
}
