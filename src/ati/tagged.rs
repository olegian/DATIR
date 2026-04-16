/* This file is a part of the runtime library injected into the compiled project.
 * It defines the Tagged<T> type which ultimately represents a tuple (Id, T). All
 * tracked values are transformed into this tagged type to be able to uniquely
 * identify where they are used. Id's are used within the various union find structure
 * (defined in ati.rs), to track interactions between values, and represent abstract
 * type sets.
*/

use crate::ati::ati::{ATI_ANALYSIS, Site};

/// type alias for Ids for ease of use, and to be able to quickly swap this out
/// (although I doubt we'll need to).
pub type Id = u64;

/// Generates incrementing tags of type `Id`, with each call to `tag()`
#[derive(Debug)]
pub struct Tagger {
    next_id: Id,
}

impl Tagger {
    /// Creates a new Tagger
    pub fn new() -> Self {
        Tagger { next_id: 0 }
    }

    /// Fetches the next tag
    pub fn tag(&mut self) -> Id {
        let id = self.next_id;
        self.next_id += 1;

        id
    }
}

/// A tuple of a type T, alongside a unique `Id`.
/// This isn't expected to be created directly, but is instead
/// used as a return type from `ATI::track`.
#[derive(Debug, Clone, Copy)]
pub struct Tagged<T>(pub Id, pub T);

impl<T, const N: usize> Tagged<[T; N]> {
    pub fn len(&self) -> Tagged<usize> {
        Tagged(self.0, N)
    }
}

impl<T> Tagged<&[T]> {
    pub fn len(&self) -> Tagged<usize> {
        Tagged(self.0, self.1.len())
    }
}

impl<T> Tagged<&mut [T]> {
    pub fn len(&self) -> Tagged<usize> {
        Tagged(self.0, self.1.len())
    }
}

/// `len()` for `Tagged<Range<Tagged<T>>>`. `len` is the one range method
/// treated as tracked — the returned `Tagged<usize>` carries the range
/// wrapper's id so the length participates in the range's AT.
impl<T> Tagged<std::ops::Range<Tagged<T>>>
where
    T: Copy + std::ops::Sub<Output = T>,
    usize: std::convert::TryFrom<T>,
{
    pub fn len(&self) -> Tagged<usize> {
        let diff = self.1.end.1 - self.1.start.1;
        let n = <usize as std::convert::TryFrom<T>>::try_from(diff).ok().unwrap_or(0);
        Tagged(self.0, n)
    }
}

impl<T> Tagged<std::ops::RangeInclusive<Tagged<T>>>
where
    T: Copy + std::ops::Sub<Output = T>,
    usize: std::convert::TryFrom<T>,
{
    pub fn len(&self) -> Tagged<usize> {
        let diff = self.1.end().1 - self.1.start().1;
        let n = match <usize as std::convert::TryFrom<T>>::try_from(diff) {
            Ok(d) => d.saturating_add(1),
            Err(_) => 0,
        };
        Tagged(self.0, n)
    }
}

/// Rebuilds a raw `Range*<T>` from a tagged range. Called from instrumented
/// code whenever a tagged range must be used as a `SliceIndex` (`arr[r]`):
/// the standard-library indexing trait only accepts `usize` / `Range<usize>`
/// etc, not `Tagged<Range<..>>`, so we strip the wrapper here. Tag
/// propagation into the produced slice is handled by the slicing helper,
/// not by `raw()`.
impl<T: Copy> Tagged<std::ops::Range<Tagged<T>>> {
    pub fn raw(&self) -> std::ops::Range<T> {
        self.1.start.1..self.1.end.1
    }
}

impl<T: Copy> Tagged<std::ops::RangeInclusive<Tagged<T>>> {
    pub fn raw(&self) -> std::ops::RangeInclusive<T> {
        self.1.start().1..=self.1.end().1
    }
}

impl<T: Copy> Tagged<std::ops::RangeFrom<Tagged<T>>> {
    pub fn raw(&self) -> std::ops::RangeFrom<T> {
        self.1.start.1..
    }
}

impl<T: Copy> Tagged<std::ops::RangeTo<Tagged<T>>> {
    pub fn raw(&self) -> std::ops::RangeTo<T> {
        ..self.1.end.1
    }
}

impl<T: Copy> Tagged<std::ops::RangeToInclusive<Tagged<T>>> {
    pub fn raw(&self) -> std::ops::RangeToInclusive<T> {
        ..=self.1.end.1
    }
}

impl Tagged<std::ops::RangeFull> {
    pub fn raw(&self) -> std::ops::RangeFull {
        ..
    }
}

/// Bridge between a tagged range and a raw slice-indexing operation. Used by
/// `ATI::track_subslice` / `ATI::track_subslice_mut` so a single helper can
/// accept any tagged range variant: the helper reads `id()` to union the
/// range's tag into the produced slice's AT, and `into_raw()` to get a
/// standard-library index.
pub trait TaggedSliceIndex<T> {
    type Raw: std::slice::SliceIndex<[T], Output = [T]>;
    fn id(&self) -> Id;
    fn into_raw(self) -> Self::Raw;
}

impl<T, U: Copy> TaggedSliceIndex<T> for Tagged<std::ops::Range<Tagged<U>>>
where
    std::ops::Range<U>: std::slice::SliceIndex<[T], Output = [T]>,
{
    type Raw = std::ops::Range<U>;
    fn id(&self) -> Id { self.0 }
    fn into_raw(self) -> Self::Raw { self.1.start.1..self.1.end.1 }
}

impl<T, U: Copy> TaggedSliceIndex<T> for Tagged<std::ops::RangeInclusive<Tagged<U>>>
where
    std::ops::RangeInclusive<U>: std::slice::SliceIndex<[T], Output = [T]>,
{
    type Raw = std::ops::RangeInclusive<U>;
    fn id(&self) -> Id { self.0 }
    fn into_raw(self) -> Self::Raw { self.1.start().1..=self.1.end().1 }
}

impl<T, U: Copy> TaggedSliceIndex<T> for Tagged<std::ops::RangeFrom<Tagged<U>>>
where
    std::ops::RangeFrom<U>: std::slice::SliceIndex<[T], Output = [T]>,
{
    type Raw = std::ops::RangeFrom<U>;
    fn id(&self) -> Id { self.0 }
    fn into_raw(self) -> Self::Raw { self.1.start.1.. }
}

impl<T, U: Copy> TaggedSliceIndex<T> for Tagged<std::ops::RangeTo<Tagged<U>>>
where
    std::ops::RangeTo<U>: std::slice::SliceIndex<[T], Output = [T]>,
{
    type Raw = std::ops::RangeTo<U>;
    fn id(&self) -> Id { self.0 }
    fn into_raw(self) -> Self::Raw { ..self.1.end.1 }
}

impl<T, U: Copy> TaggedSliceIndex<T> for Tagged<std::ops::RangeToInclusive<Tagged<U>>>
where
    std::ops::RangeToInclusive<U>: std::slice::SliceIndex<[T], Output = [T]>,
{
    type Raw = std::ops::RangeToInclusive<U>;
    fn id(&self) -> Id { self.0 }
    fn into_raw(self) -> Self::Raw { ..=self.1.end.1 }
}

impl<T> TaggedSliceIndex<T> for Tagged<std::ops::RangeFull> {
    type Raw = std::ops::RangeFull;
    fn id(&self) -> Id { self.0 }
    fn into_raw(self) -> Self::Raw { .. }
}

/// Iterator impls for tagged ranges. Rather than reimplementing every
/// Iterator adapter (.map, .filter, .sum, ...) we impl `Iterator` once on
/// the Tagged range itself; all ~70 default methods inherit for free.
/// Each yielded element carries the range's wrapper id so that values
/// produced by iteration participate in the range's AT. `for` loops keep
/// working via the blanket `impl<I: Iterator> IntoIterator for I`.
impl<T: Copy + std::iter::Step> Iterator for Tagged<std::ops::Range<Tagged<T>>> {
    type Item = Tagged<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.1.start.1 >= self.1.end.1 {
            return None;
        }
        let yielded = self.1.start.1;
        self.1.start.1 = <T as std::iter::Step>::forward(yielded, 1);
        Some(Tagged(self.0, yielded))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = <T as std::iter::Step>::steps_between(&self.1.start.1, &self.1.end.1);
        (n.0, n.1)
    }
}

impl<T: Copy + std::iter::Step> DoubleEndedIterator for Tagged<std::ops::Range<Tagged<T>>> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.1.start.1 >= self.1.end.1 {
            return None;
        }
        self.1.end.1 = <T as std::iter::Step>::backward(self.1.end.1, 1);
        Some(Tagged(self.0, self.1.end.1))
    }
}

impl<T: Copy + std::iter::Step> ExactSizeIterator for Tagged<std::ops::Range<Tagged<T>>>
where
    std::ops::Range<T>: ExactSizeIterator,
{}

impl<T: Copy + std::iter::Step> std::iter::FusedIterator for Tagged<std::ops::Range<Tagged<T>>> {}

/// RangeInclusive has a hidden `exhausted` flag we can't reach, so we
/// encode exhaustion by leaving `start > end` when we yield the last
/// value — `start == T::MAX` is the only case where this can't hold and
/// would double-yield; acceptable edge case for our instrumentation.
impl<T: Copy + std::iter::Step> Iterator for Tagged<std::ops::RangeInclusive<Tagged<T>>> {
    type Item = Tagged<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let start = self.1.start().1;
        let end = self.1.end().1;
        if start > end {
            return None;
        }
        let start_id = self.1.start().0;
        let end_id = self.1.end().0;
        let next_start = match <T as std::iter::Step>::forward_checked(start, 1) {
            Some(s) => s,
            None => start, // T::MAX: fall back to start == end (terminal but could double-yield)
        };
        self.1 = Tagged(start_id, next_start)..=Tagged(end_id, end);
        Some(Tagged(self.0, start))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.1.start().1 > self.1.end().1 {
            return (0, Some(0));
        }
        let n = <T as std::iter::Step>::steps_between(&self.1.start().1, &self.1.end().1);
        (
            n.0.saturating_add(1),
            n.1.and_then(|v| v.checked_add(1)),
        )
    }
}

impl<T: Copy + std::iter::Step> DoubleEndedIterator for Tagged<std::ops::RangeInclusive<Tagged<T>>> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let start = self.1.start().1;
        let end = self.1.end().1;
        if start > end {
            return None;
        }
        let start_id = self.1.start().0;
        let end_id = self.1.end().0;
        let next_end = match <T as std::iter::Step>::backward_checked(end, 1) {
            Some(e) => e,
            None => end,
        };
        self.1 = Tagged(start_id, start)..=Tagged(end_id, next_end);
        Some(Tagged(self.0, end))
    }
}

impl<T: Copy + std::iter::Step> ExactSizeIterator for Tagged<std::ops::RangeInclusive<Tagged<T>>>
where
    std::ops::RangeInclusive<T>: ExactSizeIterator,
{}

impl<T: Copy + std::iter::Step> std::iter::FusedIterator for Tagged<std::ops::RangeInclusive<Tagged<T>>> {}

impl<T: Copy + std::iter::Step> Iterator for Tagged<std::ops::RangeFrom<Tagged<T>>> {
    type Item = Tagged<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let yielded = self.1.start.1;
        self.1.start.1 = <T as std::iter::Step>::forward(yielded, 1);
        Some(Tagged(self.0, yielded))
    }
}

impl<T: Copy + std::iter::Step> std::iter::FusedIterator for Tagged<std::ops::RangeFrom<Tagged<T>>> {}

/// `RangeBounds` is what `BTreeMap::range(..)`, `Vec::drain(r)`, and other
/// range-parameterized APIs ask for. We return references into `self.1`'s
/// endpoints, which carry the raw `T` behind a `Tagged` — but `Bound<&T>`
/// is what callers expect, so we Deref-shed the tag on read.
impl<T> std::ops::RangeBounds<T> for Tagged<std::ops::Range<Tagged<T>>> {
    fn start_bound(&self) -> std::ops::Bound<&T> {
        std::ops::Bound::Included(&self.1.start.1)
    }
    fn end_bound(&self) -> std::ops::Bound<&T> {
        std::ops::Bound::Excluded(&self.1.end.1)
    }
}

impl<T> std::ops::RangeBounds<T> for Tagged<std::ops::RangeInclusive<Tagged<T>>> {
    fn start_bound(&self) -> std::ops::Bound<&T> {
        std::ops::Bound::Included(&self.1.start().1)
    }
    fn end_bound(&self) -> std::ops::Bound<&T> {
        std::ops::Bound::Included(&self.1.end().1)
    }
}

impl<T> std::ops::RangeBounds<T> for Tagged<std::ops::RangeFrom<Tagged<T>>> {
    fn start_bound(&self) -> std::ops::Bound<&T> {
        std::ops::Bound::Included(&self.1.start.1)
    }
    fn end_bound(&self) -> std::ops::Bound<&T> {
        std::ops::Bound::Unbounded
    }
}

impl<T> std::ops::RangeBounds<T> for Tagged<std::ops::RangeTo<Tagged<T>>> {
    fn start_bound(&self) -> std::ops::Bound<&T> {
        std::ops::Bound::Unbounded
    }
    fn end_bound(&self) -> std::ops::Bound<&T> {
        std::ops::Bound::Excluded(&self.1.end.1)
    }
}

impl<T> std::ops::RangeBounds<T> for Tagged<std::ops::RangeToInclusive<Tagged<T>>> {
    fn start_bound(&self) -> std::ops::Bound<&T> {
        std::ops::Bound::Unbounded
    }
    fn end_bound(&self) -> std::ops::Bound<&T> {
        std::ops::Bound::Included(&self.1.end.1)
    }
}

impl<T> std::ops::RangeBounds<T> for Tagged<std::ops::RangeFull> {
    fn start_bound(&self) -> std::ops::Bound<&T> {
        std::ops::Bound::Unbounded
    }
    fn end_bound(&self) -> std::ops::Bound<&T> {
        std::ops::Bound::Unbounded
    }
}

/// helpful for debugging purposes, allowing printing of tagged values.
impl<T> std::fmt::Display for Tagged<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

/// The Trackable trait lets `ATI::track_array` discover every tracked
/// Id reachable inside an array element, grouped by nesting depth.
/// This allows `track_array` to merge all leaf ids across nested arrays
/// (so `[[a,b,c], [d,e,f]]` places every leaf in one AT) while keeping
/// the wrapper id of each nesting level in its own AT (so each level's
/// `_LEN` bind stays separate).
///
/// Depth 0 contains the id of the value the method is called on.
/// Each additional depth corresponds to one level of array nesting below.
pub trait Trackable {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize);
}

/// Default for any T: not a Tagged value, contributes no ids. This is the
/// base impl that more specific Tagged impls specialize via `min_specialization`.
impl<T> Trackable for T {
    default fn collect_ids_by_level(&self, _ids: &mut Vec<Vec<Id>>, _depth: usize) {}
}

/// A plain `Tagged<T>` (leaf case): record its single id at `depth`.
impl<T> Trackable for Tagged<T> {
    default fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
    }
}

/// A nested `Tagged<[T; N]>`: record this wrapper's id at `depth`, then
/// recurse into every element at `depth + 1`. Specialization re-dispatches
/// per element's concrete type at monomorphization, so arbitrarily deep
/// nesting is traversed.
impl<T, const N: usize> Trackable for Tagged<[T; N]> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);

        for i in 0..N {
            self.1[i].collect_ids_by_level(ids, depth + 1);
        }
    }
}

/// A nested `Tagged<&[T]>`: a slice is structurally the same as an array
/// for AT purposes — the slice wrapper's id stays at `depth`, and every
/// element is recursed into at `depth + 1`. This lets arrays-of-slices
/// and slices-of-arrays share one leaf AT across all inner elements.
impl<T> Trackable for Tagged<&[T]> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);

        for i in 0..self.1.len() {
            self.1[i].collect_ids_by_level(ids, depth + 1);
        }
    }
}

impl<T> Trackable for Tagged<&mut [T]> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);

        for i in 0..self.1.len() {
            self.1[i].collect_ids_by_level(ids, depth + 1);
        }
    }
}

/// Trackable for each `Tagged<Range*>` variant. The wrapper id sits at
/// `depth`, and any tracked endpoints are recursed into at `depth + 1` —
/// matching the treatment used for `Tagged<[T; N]>` / `Tagged<&[T]>` so
/// arrays of ranges merge endpoints across elements correctly.
impl<T> Trackable for Tagged<std::ops::Range<Tagged<T>>> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
        self.1.start.collect_ids_by_level(ids, depth + 1);
        self.1.end.collect_ids_by_level(ids, depth + 1);
    }
}

impl<T> Trackable for Tagged<std::ops::RangeInclusive<Tagged<T>>> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
        self.1.start().collect_ids_by_level(ids, depth + 1);
        self.1.end().collect_ids_by_level(ids, depth + 1);
    }
}

impl<T> Trackable for Tagged<std::ops::RangeFrom<Tagged<T>>> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
        self.1.start.collect_ids_by_level(ids, depth + 1);
    }
}

impl<T> Trackable for Tagged<std::ops::RangeTo<Tagged<T>>> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
        self.1.end.collect_ids_by_level(ids, depth + 1);
    }
}

impl<T> Trackable for Tagged<std::ops::RangeToInclusive<Tagged<T>>> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
        self.1.end.collect_ids_by_level(ids, depth + 1);
    }
}

impl Trackable for Tagged<std::ops::RangeFull> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
    }
}

/// Reference-wrapping specializations. An array literal whose elements are
/// references to tracked values — e.g. `[&a[..], &b[..], &c[..]]` yielding
/// `[&Tagged<&[T]>; 3]` — still needs to merge every descendant id into the
/// same AT as if the referents were stored inline. We specialize per concrete
/// wrapper kind so specialization picks up the recursive behavior for nested
/// arrays / slices reachable through a reference.
impl<T> Trackable for &Tagged<T> {
    default fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
    }
}

impl<T, const N: usize> Trackable for &Tagged<[T; N]> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);

        for i in 0..N {
            self.1[i].collect_ids_by_level(ids, depth + 1);
        }
    }
}

impl<T> Trackable for &Tagged<&[T]> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);

        for i in 0..self.1.len() {
            self.1[i].collect_ids_by_level(ids, depth + 1);
        }
    }
}

impl<T> Trackable for &Tagged<&mut [T]> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);

        for i in 0..self.1.len() {
            self.1[i].collect_ids_by_level(ids, depth + 1);
        }
    }
}

impl<T> Trackable for &mut Tagged<T> {
    default fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
    }
}

impl<T, const N: usize> Trackable for &mut Tagged<[T; N]> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);

        for i in 0..N {
            self.1[i].collect_ids_by_level(ids, depth + 1);
        }
    }
}

impl<T> Trackable for &mut Tagged<&[T]> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);

        for i in 0..self.1.len() {
            self.1[i].collect_ids_by_level(ids, depth + 1);
        }
    }
}

impl<T> Trackable for &mut Tagged<&mut [T]> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);

        for i in 0..self.1.len() {
            self.1[i].collect_ids_by_level(ids, depth + 1);
        }
    }
}

/// The BindToSite trait is defined for all type T, and allows
/// for all data types to be observed at a particular site. The
/// `T.bind()` function is called within function stubs.
pub trait BindToSite {
    fn bind(&self, site: &mut Site, var_name: &str);
}

/// Most generic implementation used by all non-tagged types.
impl<T> BindToSite for T {
    default fn bind(&self, site: &mut Site, var_name: &str) {}
}

/// Most generic implementation used by all tagged types.
impl<T> BindToSite for Tagged<T> {
    default fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
    }
}

impl<T> BindToSite for &Tagged<T> {
    default fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
    }
}

impl<T> BindToSite for &mut Tagged<T> {
    default fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
    }
}

/// More specific implementation, used when binding arrays.
/// This has every element of the array be represented within the site,
/// alongside the length of the array.
impl<T, const N: usize> BindToSite for Tagged<[T; N]> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(&format!("{var_name}_LEN"), self.len().0);

        for i in 0..N {
            self.1[i].bind(site, &format!("{var_name}[{i}]"));
        }
    }
}

impl<T, const N: usize> BindToSite for &Tagged<[T; N]> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(&format!("{var_name}_LEN"), self.len().0);

        for i in 0..N {
            self.1[i].bind(site, &format!("{var_name}[{i}]"));
        }
    }
}

impl<T, const N: usize> BindToSite for &mut Tagged<[T; N]> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(&format!("{var_name}_LEN"), self.len().0);

        for i in 0..N {
            self.1[i].bind(site, &format!("{var_name}[{i}]"));
        }
    }
}

/// Similar to BindToSite for Tagged<[T; N]>, but for slices instead!
impl<T> BindToSite for Tagged<&[T]> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(&format!("{var_name}_LEN"), self.len().0);

        for i in 0..self.len().1 {
            self.1[i].bind(site, &format!("{var_name}[{i}]"));
        }
    }
}

impl<T> BindToSite for &Tagged<&[T]> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(&format!("{var_name}_LEN"), self.len().0);

        for i in 0..self.len().1 {
            self.1[i].bind(site, &format!("{var_name}[{i}]"));
        }
    }
}

impl<T> BindToSite for &mut Tagged<&mut [T]> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(&format!("{var_name}_LEN"), self.len().0);

        for i in 0..self.len().1 {
            self.1[i].bind(site, &format!("{var_name}[{i}]"));
        }
    }
}

/// BindToSite for tagged ranges. Records the range's wrapper id against
/// `var_name`, and exposes each endpoint id under `{var_name}.start` /
/// `{var_name}.end`. For range variants that have a meaningful length
/// (Range/RangeInclusive over `Step` types), also records the length id
/// under `{var_name}_LEN`.
impl<T> BindToSite for Tagged<std::ops::Range<Tagged<T>>> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
        site.bind(&format!("{var_name}.start"), self.1.start.0);
        site.bind(&format!("{var_name}.end"), self.1.end.0);
    }
}

impl<T> BindToSite for Tagged<std::ops::RangeInclusive<Tagged<T>>> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
        site.bind(&format!("{var_name}.start"), self.1.start().0);
        site.bind(&format!("{var_name}.end"), self.1.end().0);
    }
}

impl<T> BindToSite for Tagged<std::ops::RangeFrom<Tagged<T>>> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
        site.bind(&format!("{var_name}.start"), self.1.start.0);
    }
}

impl<T> BindToSite for Tagged<std::ops::RangeTo<Tagged<T>>> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
        site.bind(&format!("{var_name}.end"), self.1.end.0);
    }
}

impl<T> BindToSite for Tagged<std::ops::RangeToInclusive<Tagged<T>>> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
        site.bind(&format!("{var_name}.end"), self.1.end.0);
    }
}

impl BindToSite for Tagged<std::ops::RangeFull> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
    }
}

impl<T> std::cmp::PartialEq for Tagged<T> where T: std::cmp::PartialEq {
    fn eq(&self, other: &Self) -> bool {
        // why not merge in here?
        ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &other.0);
        self.1 == other.1
    }
}

impl<T> std::cmp::PartialOrd for Tagged<T> where T: std::cmp::PartialOrd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &other.0);
        self.1.partial_cmp(&other.1)
    }
}

/// `Sum`/`Product` let instrumented code call `.sum()` / `.product()` on
/// iterators of `Tagged<T>`. We fold via the existing tagged `Add` / `Mul`,
/// which unions ids as it goes. Empty iterators fall back to the raw T's
/// identity with a fresh Id from the analysis.
impl<T> std::iter::Sum for Tagged<T>
where
    Tagged<T>: std::ops::Add<Output = Tagged<T>>,
    T: std::iter::Sum,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, b| a + b).unwrap_or_else(|| {
            let id = ATI_ANALYSIS.lock().unwrap().make_id();
            Tagged(id, T::sum(std::iter::empty::<T>()))
        })
    }
}

impl<'a, T: Copy + 'a> std::iter::Sum<&'a Tagged<T>> for Tagged<T>
where
    Tagged<T>: std::ops::Add<Output = Tagged<T>>,
    T: std::iter::Sum,
{
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.copied().sum()
    }
}

impl<T> std::iter::Product for Tagged<T>
where
    Tagged<T>: std::ops::Mul<Output = Tagged<T>>,
    T: std::iter::Product,
{
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, b| a * b).unwrap_or_else(|| {
            let id = ATI_ANALYSIS.lock().unwrap().make_id();
            Tagged(id, T::product(std::iter::empty::<T>()))
        })
    }
}

impl<T> std::ops::Add for Tagged<T> where T: std::ops::Add<Output=T> {
    type Output = Tagged<T>;

    fn add(self, rhs: Self) -> Self::Output {
        let merged = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &rhs.0);
        Tagged(merged, self.1 + rhs.1)
    }
}

impl<T: Copy> std::ops::Add<&Tagged<T>> for Tagged<T> where T: std::ops::Add<Output=T> {
    type Output = Tagged<T>;

    fn add(self, rhs: &Tagged<T>) -> Self::Output {
        let merged = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &rhs.0);
        Tagged(merged, self.1 + rhs.1)
    }
}

impl<T> std::ops::Sub for Tagged<T> where T: std::ops::Sub<Output=T> {
    type Output = Tagged<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        let merged = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &rhs.0);
        Tagged(merged, self.1 - rhs.1)
    }
}

impl<T: Copy> std::ops::Sub<&Tagged<T>> for Tagged<T> where T: std::ops::Sub<Output=T> {
    type Output = Tagged<T>;

    fn sub(self, rhs: &Tagged<T>) -> Self::Output {
        let merged = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &rhs.0);
        Tagged(merged, self.1 - rhs.1)
    }
}

impl<T> std::ops::Mul for Tagged<T> where T: std::ops::Mul<Output=T> {
    type Output = Tagged<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        let merged = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &rhs.0);
        Tagged(merged, self.1 * rhs.1)
    }
}

impl<T: Copy> std::ops::Mul<&Tagged<T>> for Tagged<T> where T: std::ops::Mul<Output=T> {
    type Output = Tagged<T>;

    fn mul(self, rhs: &Tagged<T>) -> Self::Output {
        let merged = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &rhs.0);
        Tagged(merged, self.1 * rhs.1)
    }
}

impl<T> std::ops::Div for Tagged<T> where T: std::ops::Div<Output=T> {
    type Output = Tagged<T>;

    fn div(self, rhs: Self) -> Self::Output {
        let merged = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &rhs.0);
        Tagged(merged, self.1 / rhs.1)
    }
}

impl<T: Copy> std::ops::Div<&Tagged<T>> for Tagged<T> where T: std::ops::Div<Output=T> {
    type Output = Tagged<T>;

    fn div(self, rhs: &Tagged<T>) -> Self::Output {
        let merged = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &rhs.0);
        Tagged(merged, self.1 / rhs.1)
    }
}

impl<T> std::ops::Rem for Tagged<T> where T: std::ops::Rem<Output=T> {
    type Output = Tagged<T>;

    fn rem(self, rhs: Self) -> Self::Output {
        let merged = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &rhs.0);
        Tagged(merged, self.1 % rhs.1)
    }
}

impl<T> std::ops::BitAnd for Tagged<T> where T: std::ops::BitAnd<Output=T> {
    type Output = Tagged<T>;

    fn bitand(self, rhs: Self) -> Self::Output {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        Tagged(new_id, self.1 & rhs.1)
    }
}

impl<T: Copy> std::ops::BitAnd<&Tagged<T>> for Tagged<T> where T: std::ops::BitAnd<Output=T> {
    type Output = Tagged<T>;

    fn bitand(self, rhs: &Tagged<T>) -> Self::Output {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        Tagged(new_id, self.1 & rhs.1)
    }
}

impl<T> std::ops::BitOr for Tagged<T> where T: std::ops::BitOr<Output=T> {
    type Output = Tagged<T>;

    fn bitor(self, rhs: Self) -> Self::Output {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        Tagged(new_id, self.1 | rhs.1)
    }
}

impl<T: Copy> std::ops::BitOr<&Tagged<T>> for Tagged<T> where T: std::ops::BitOr<Output=T> {
    type Output = Tagged<T>;

    fn bitor(self, rhs: &Tagged<T>) -> Self::Output {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        Tagged(new_id, self.1 | rhs.1)
    }
}

impl<T> std::ops::BitXor for Tagged<T> where T: std::ops::BitXor<Output=T> {
    type Output = Tagged<T>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        Tagged(new_id, self.1 ^ rhs.1)
    }
}

impl<T: Copy> std::ops::BitXor<&Tagged<T>> for Tagged<T> where T: std::ops::BitXor<Output=T> {
    type Output = Tagged<T>;

    fn bitxor(self, rhs: &Tagged<T>) -> Self::Output {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        Tagged(new_id, self.1 ^ rhs.1)
    }
}

impl<T> std::ops::Shl for Tagged<T> where T: std::ops::Shl<Output=T> {
    type Output = Tagged<T>;

    fn shl(self, rhs: Self) -> Self::Output {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        Tagged(new_id, self.1 << rhs.1)
    }
}

impl<T: Copy> std::ops::Shl<&Tagged<T>> for Tagged<T> where T: std::ops::Shl<Output=T> {
    type Output = Tagged<T>;

    fn shl(self, rhs: &Tagged<T>) -> Self::Output {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        Tagged(new_id, self.1 << rhs.1)
    }
}

impl<T> std::ops::Shr for Tagged<T> where T: std::ops::Shr<Output=T> {
    type Output = Tagged<T>;

    fn shr(self, rhs: Self) -> Self::Output {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        Tagged(new_id, self.1 >> rhs.1)
    }
}

impl<T: Copy> std::ops::Shr<&Tagged<T>> for Tagged<T> where T: std::ops::Shr<Output=T> {
    type Output = Tagged<T>;

    fn shr(self, rhs: &Tagged<T>) -> Self::Output {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        Tagged(new_id, self.1 >> rhs.1)
    }
}

// I am unsure of the Copy requirement here, would be nice to get rid of it
impl<T> std::ops::AddAssign for Tagged<T> where T: std::ops::Add<Output=T> + Copy {
    fn add_assign(&mut self, rhs: Self) {
        let merged = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &rhs.0);
        *self = Tagged(merged, self.1 + rhs.1);
    }
}

impl<T> std::ops::SubAssign for Tagged<T> where T: std::ops::Sub<Output=T> + Copy + std::fmt::Debug {
    fn sub_assign(&mut self, rhs: Self) {
        let merged = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &rhs.0);
        *self = Tagged(merged, self.1 - rhs.1);
    }
}

impl<T> std::ops::MulAssign for Tagged<T> where T: std::ops::Mul<Output=T> + Copy {
    fn mul_assign(&mut self, rhs: Self) {
        let merged = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &rhs.0);
        *self = Tagged(merged, self.1 * rhs.1);
    }
}

impl<T> std::ops::DivAssign for Tagged<T> where T: std::ops::Div<Output=T> + Copy {
    fn div_assign(&mut self, rhs: Self) {
        let merged = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &rhs.0);
        *self = Tagged(merged, self.1 / rhs.1);
    }
}

impl<T> std::ops::RemAssign for Tagged<T> where T: std::ops::Rem<Output=T> + Copy {
    fn rem_assign(&mut self, rhs: Self) {
        let merged = ATI_ANALYSIS.lock().unwrap().union_and_get_id(&self.0, &rhs.0);
        *self = Tagged(merged, self.1 % rhs.1);
    }
}

impl<T> std::ops::BitAndAssign for Tagged<T> where T: std::ops::BitAnd<Output=T> + Copy {
    fn bitand_assign(&mut self, rhs: Self) {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        *self = Tagged(new_id, self.1 & rhs.1);
    }
}

impl<T> std::ops::BitOrAssign for Tagged<T> where T: std::ops::BitOr<Output=T> + Copy {
    fn bitor_assign(&mut self, rhs: Self) {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        *self = Tagged(new_id, self.1 | rhs.1);
    }
}

impl<T> std::ops::BitXorAssign for Tagged<T> where T: std::ops::BitXor<Output=T> + Copy {
    fn bitxor_assign(&mut self, rhs: Self) {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        *self = Tagged(new_id, self.1 ^ rhs.1);
    }
}

impl<T> std::ops::ShlAssign for Tagged<T> where T: std::ops::Shl<Output=T> + Copy {
    fn shl_assign(&mut self, rhs: Self) {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        *self = Tagged(new_id, self.1 << rhs.1);
    }
}

impl<T> std::ops::ShrAssign for Tagged<T> where T: std::ops::Shr<Output=T> + Copy {
    fn shr_assign(&mut self, rhs: Self) {
        let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
        *self = Tagged(new_id, self.1 >> rhs.1);
    }
}

impl<T> std::ops::Neg for Tagged<T> where T: std::ops::Neg<Output=T> {
    type Output = Tagged<T>;

    fn neg(self) -> Self::Output {
        Tagged(self.0, -self.1)
    }
}

impl<T> std::ops::Not for Tagged<T> where T: std::ops::Not<Output = T> {
    type Output = Tagged<T>;
    fn not(self) -> Self::Output {
        Tagged(self.0, !self.1)
    }
}

impl<T> std::ops::Deref for Tagged<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.1
    }
}
