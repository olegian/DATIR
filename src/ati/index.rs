use crate::ati::{
    ati::ATI_ANALYSIS,
    tagged::{
        Id, Tagged, TaggedArray, TaggedRange, TaggedRangeFrom, TaggedRangeFull, TaggedRangeInclusive, TaggedRangeTo, TaggedRangeToInclusive, TaggedSlice, TaggedSliceMut
    },
};
// ==============    REGULAR INDEXING   =================
// [T; N]
impl<Idx, T, const N: usize> std::ops::Index<Tagged<Idx>> for TaggedArray<T, N>
where
    [T; N]: std::ops::Index<Idx, Output=T>,
{
    type Output = T;

    fn index(&self, index: Tagged<Idx>) -> &Self::Output {
        ATI_ANALYSIS
            .lock()
            .unwrap()
            .union_and_get_id(&self.0, &index.0);
        &self.1[index.1]
    }
}
impl<Idx, T, const N: usize> std::ops::IndexMut<Tagged<Idx>> for TaggedArray<T, N>
where
    [T; N]: std::ops::IndexMut<Idx, Output=T>
{
    fn index_mut(&mut self, index: Tagged<Idx>) -> &mut Self::Output {
        ATI_ANALYSIS
            .lock()
            .unwrap()
            .union_and_get_id(&self.0, &index.0);
        &mut self.1[index.1]
    }
}

// &[T]
impl<'slice, Idx, T> std::ops::Index<Tagged<Idx>> for TaggedSlice<'slice, T>
where
    [T]: std::ops::Index<Idx, Output=T>,
{
    type Output = T;

    fn index(&self, index: Tagged<Idx>) -> &Self::Output {
        ATI_ANALYSIS
            .lock()
            .unwrap()
            .union_and_get_id(&self.0, &index.0);
        &self.1[index.1]
    }
}

// &mut [T]
impl<'slice, Idx, T> std::ops::Index<Tagged<Idx>> for TaggedSliceMut<'slice, T>
where
    [T]: std::ops::Index<Idx, Output=T>,
{
    type Output = T;

    fn index(&self, index: Tagged<Idx>) -> &Self::Output {
        ATI_ANALYSIS
            .lock()
            .unwrap()
            .union_and_get_id(&self.0, &index.0);
        &self.1[index.1]
    }
}
impl<'slice, Idx, T> std::ops::IndexMut<Tagged<Idx>> for TaggedSliceMut<'slice, T>
where
    [T]: std::ops::IndexMut<Idx, Output=T>,
{
    fn index_mut(&mut self, index: Tagged<Idx>) -> &mut Self::Output {
        ATI_ANALYSIS
            .lock()
            .unwrap()
            .union_and_get_id(&self.0, &index.0);
        &mut self.1[index.1]
    }
}

// ==============    SLICE INDEXING   =================

/// Implementors of this trait are tagged-ranges, used as indexes that can
/// access some collection e.g. in `array[range]`, `range`'s type must implement this trait.
/// This allows for the Index operation to utilize the into_raw method to
/// convert the tagged range into a simple range, after merging appropriate ids.
pub trait TaggedSliceIndex<T> {
    type Raw: std::slice::SliceIndex<[T], Output = [T]>;
    fn id(&self) -> Id;
    fn into_raw(self) -> Self::Raw;
}

impl<Idx, T: Copy> TaggedSliceIndex<Idx> for TaggedRange<T>
where
    std::ops::Range<T>: std::slice::SliceIndex<[Idx], Output = [Idx]>,
{
    type Raw = std::ops::Range<T>;
    fn id(&self) -> Id { self.0 }
    fn into_raw(self) -> Self::Raw {
        self.1.start.1..self.1.end.1
    }
}
impl<Idx, T: Copy> TaggedSliceIndex<Idx> for TaggedRangeInclusive<T>
where
    std::ops::RangeInclusive<T>: std::slice::SliceIndex<[Idx], Output = [Idx]>,
{
    type Raw = std::ops::RangeInclusive<T>;
    fn id(&self) -> Id { self.0 }
    fn into_raw(self) -> Self::Raw {
        self.1.start().1..=self.1.end().1
    }
}
impl<Idx, T: Copy> TaggedSliceIndex<Idx> for TaggedRangeFrom<T>
where
    std::ops::RangeFrom<T>: std::slice::SliceIndex<[Idx], Output = [Idx]>,
{
    type Raw = std::ops::RangeFrom<T>;
    fn id(&self) -> Id { self.0 }
    fn into_raw(self) -> Self::Raw {
        self.1.start.1..
    }
}
impl<Idx, T: Copy> TaggedSliceIndex<Idx> for TaggedRangeTo<T>
where
    std::ops::RangeTo<T>: std::slice::SliceIndex<[Idx], Output = [Idx]>,
{
    type Raw = std::ops::RangeTo<T>;
    fn id(&self) -> Id { self.0 }
    fn into_raw(self) -> Self::Raw {
        ..self.1.end.1
    }
}
impl<Idx, T: Copy> TaggedSliceIndex<Idx> for TaggedRangeToInclusive<T>
where
    std::ops::RangeToInclusive<T>: std::slice::SliceIndex<[Idx], Output = [Idx]>,
{
    type Raw = std::ops::RangeToInclusive<T>;
    fn id(&self) -> Id { self.0 }
    fn into_raw(self) -> Self::Raw {
        ..=self.1.end.1
    }
}
impl<T> TaggedSliceIndex<T> for TaggedRangeFull {
    type Raw = std::ops::RangeFull;
    fn id(&self) -> Id { self.0 }
    fn into_raw(self) -> Self::Raw {
        ..
    }
}

/// Implementors of this trait are receivers of slice operations, e.g. `array[range]`
/// array implements TaggedSliceable. This allows calling .raw_subslice(range.into_raw())
/// to slice into any slice/array with any tagged range.
pub trait TaggedSliceable<'a, T> {
    fn id(&self) -> Id;
    fn raw_subslice<'b, R>(&'b self, range: R) -> &'b [T]
    where
        R: std::slice::SliceIndex<[T], Output = [T]>;
    fn raw_subslice_mut<'b, R>(&'b mut self, range: R) -> &'b mut [T]
    where
        R: std::slice::SliceIndex<[T], Output = [T]>;
}

// allows slicing [T; N]
impl<'a, T, const N: usize> TaggedSliceable<'a, T> for TaggedArray<T, N> {
    fn id(&self) -> Id { self.0 }
    fn raw_subslice<'b, R>(&'b self, range: R) -> &'b [T]
    where
        R: std::slice::SliceIndex<[T], Output = [T]>,
    {
        &self.1[range]
    }
    fn raw_subslice_mut<'b, R>(&'b mut self, range: R) -> &'b mut [T]
    where
        R: std::slice::SliceIndex<[T], Output = [T]>
    {
        &mut self.1[range]
    }
}

// allows slicing &[T]
impl<'a, 'slice, T> TaggedSliceable<'a, T> for TaggedSlice<'slice, T>
{
    fn id(&self) -> Id { self.0 }
    fn raw_subslice<'b, R>(&'b self, range: R) -> &'b [T]
    where
        R: std::slice::SliceIndex<[T], Output = [T]>,
    {
        &self.1[range]
    }
    fn raw_subslice_mut<'b, R>(&'b mut self, range: R) -> &'b mut [T]
    where
        R: std::slice::SliceIndex<[T], Output = [T]> 
    {
        panic!("Tried to get a mutable subslice behind an immutable slice (TaggedSlice)");
    }
}

// allows slicing &mut [T]
impl<'a, 'slice, T> TaggedSliceable<'a, T> for TaggedSliceMut<'slice, T>
{
    fn id(&self) -> Id { self.0 }
    fn raw_subslice<'b, R>(&'b self, range: R) -> &'b [T]
    where
        R: std::slice::SliceIndex<[T], Output = [T]>,
    {
        &self.1[range]
    }
    fn raw_subslice_mut<'b, R>(&'b mut self, range: R) -> &'b mut [T]
    where
        R: std::slice::SliceIndex<[T], Output = [T]> 
    {
        &mut self.1[range]
    }
}
