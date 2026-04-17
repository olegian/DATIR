use crate::ati::tagged::{Id, Tagged, TaggedArray, TaggedRange, TaggedRangeFrom, TaggedRangeFull, TaggedRangeInclusive, TaggedRangeTo, TaggedRangeToInclusive, TaggedSlice, TaggedSliceMut};

pub trait Collect {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize);
}

// Leaf cases for single values
impl<T> Collect for Tagged<T> {
    default fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
    }
}

impl<T> Collect for T {
    default fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {}
}

// [T; N]
impl<T, const N: usize> Collect for TaggedArray<T, N> {
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

// [T]
impl<'a, T> Collect for TaggedSlice<'a, T> {
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

// &mut [T]
impl<'a, T> Collect for TaggedSliceMut<'a, T> {
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

// Ranges
impl<T> Collect for TaggedRange<T> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
        self.1.start.collect_ids_by_level(ids, depth + 1);
        self.1.end.collect_ids_by_level(ids, depth + 1);
    }
}
impl<T> Collect for TaggedRangeInclusive<T> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
        self.1.start().collect_ids_by_level(ids, depth + 1);
        self.1.end().collect_ids_by_level(ids, depth + 1);
    }
}
impl<T> Collect for TaggedRangeFrom<T> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
        self.1.start.collect_ids_by_level(ids, depth + 1);
    }
}
impl<T> Collect for TaggedRangeTo<T> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
        self.1.end.collect_ids_by_level(ids, depth + 1);
    }
}
impl<T> Collect for TaggedRangeToInclusive<T> {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
        self.1.end.collect_ids_by_level(ids, depth + 1);
    }
}
impl Collect for TaggedRangeFull {
    fn collect_ids_by_level(&self, ids: &mut Vec<Vec<Id>>, depth: usize) {
        if ids.len() <= depth {
            ids.resize(depth + 1, Vec::new());
        }
        ids[depth].push(self.0);
    }
}
