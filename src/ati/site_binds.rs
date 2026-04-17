use crate::ati::ati::Site;
use crate::ati::tagged::{
    Tagged, TaggedArray, TaggedRange, TaggedRangeFrom, TaggedRangeFull, TaggedRangeInclusive,
    TaggedRangeTo, TaggedRangeToInclusive, TaggedSlice, TaggedSliceMut,
};

/// Provides a method for recursively associating a variable `self` to some
/// ATI site, with the given name. All Tagged<T>'s should implement this trait
/// Compound types have to add an implementation of this trait during compile 
/// time, to allow them to be bound to sites during runtime.
/// The implementations below are required for the atomic/primitive types.
pub trait SiteBind {
    fn bind(&self, site: &mut Site, var_name: &str);
}

// ==========================    BASIC TYPES   ===============================

/// Most generic implementation used by all non-tagged types.
/// If the type is not tagged, then there is nothing to bind to the site,
/// resulting in a no-op.
impl<T> SiteBind for T {
    default fn bind(&self, site: &mut Site, var_name: &str) {}
}

/// Most generic implementation used by all atomic tagged types (like Tagged<u32>).
/// References to these values should be treated in the same way as the owned type.
impl<T> SiteBind for Tagged<T> {
    default fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
    }
}
impl<T> SiteBind for &Tagged<T> {
    default fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
    }
}
impl<T> SiteBind for &mut Tagged<T> {
    default fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
    }
}

// ==========================    ARRAY TYPES   ===============================

/// Binding an array should associate the length and values inside the array.
/// References to these arrays should be treated in the same way as the owned array.
impl<T, const N: usize> SiteBind for TaggedArray<T, N> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(&format!("{var_name}_LEN"), self.len().0);
        for i in 0..N {
            self.1[i].bind(site, &format!("{var_name}[{i}]"));
        }
    }
}
impl<T, const N: usize> SiteBind for &TaggedArray<T, N> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        (**self).bind(site, var_name);
    }
}
impl<T, const N: usize> SiteBind for &mut TaggedArray<T, N> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        (**self).bind(site, var_name);
    }
}

// ==========================    SLICE TYPES   ===============================

/// Similar to arrays, slices should associate all contained values,
/// and the length to the site.
/// All of the following implementations seek to support:
///    &    Tagged<&    [T]>
///    &    Tagged<&mut [T]>
///    &mut Tagged<&mut [T]>
// FIXME: does coercion from &mut -> & need to cause a change in type?
impl<'a, T> SiteBind for TaggedSlice<'a, T> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(&format!("{var_name}_LEN"), self.len().0);
        for i in 0..self.len().1 {
            self.1[i].bind(site, &format!("{var_name}[{i}]"));
        }
    }
}
impl<'a, T> SiteBind for &TaggedSlice<'a, T> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        (**self).bind(site, var_name);
    }
}
impl<'a, T> SiteBind for TaggedSliceMut<'a, T> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(&format!("{var_name}_LEN"), self.len().0);
        for i in 0..self.len().1 {
            self.1[i].bind(site, &format!("{var_name}[{i}]"));
        }
    }
}
impl<'a, T> SiteBind for &TaggedSliceMut<'a, T> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        (**self).bind(site, var_name);
    }
}
impl<'a, T> SiteBind for &mut TaggedSliceMut<'a, T> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        (**self).bind(site, var_name);
    }
}

// ==========================    RANGE TYPES   ===============================

/// Im not convinced that all ranges need a start/end bind, which is separate from the outer one.
impl<T> SiteBind for TaggedRange<T> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
        site.bind(&format!("{var_name}.start"), self.1.start.0);
        site.bind(&format!("{var_name}.end"), self.1.end.0);
    }
}
impl<T> SiteBind for TaggedRangeInclusive<T> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
        site.bind(&format!("{var_name}.start"), self.1.start().0);
        site.bind(&format!("{var_name}.end"), self.1.end().0);
    }
}
impl<T> SiteBind for TaggedRangeFrom<T> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
        site.bind(&format!("{var_name}.start"), self.1.start.0);
    }
}
impl<T> SiteBind for TaggedRangeTo<T> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
        site.bind(&format!("{var_name}.end"), self.1.end.0);
    }
}
impl<T> SiteBind for TaggedRangeToInclusive<T> {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
        site.bind(&format!("{var_name}.end"), self.1.end.0);
    }
}
impl SiteBind for TaggedRangeFull {
    fn bind(&self, site: &mut Site, var_name: &str) {
        site.bind(var_name, self.0);
    }
}

// ==========================    TUPLE TYPES   ===============================
/// Implements SiteBinds for tuples, where each entry in the tuple gets bound
/// to the site.
/// This is implemented for tuples up to length 12, following the convention of
/// the standard library. If more than 12 is ever necessary, add the site binds below.
macro_rules! tuple_impl_site_bind {
    ($($idx:tt : $T:ident),+) => {
        impl<$($T),+> SiteBind for ($($T,)+) {
            fn bind(&self, site: &mut Site, var_name: &str) {
                $(
                    self.$idx.bind(site, &format!("{}.{}", var_name, stringify!($idx)));
                )+
            }
        }
        impl<$($T),+> SiteBind for &($($T,)+) {
            fn bind(&self, site: &mut Site, var_name: &str) {
                (**self).bind(site, var_name);
            }
        }
        impl<$($T),+> SiteBind for &mut ($($T,)+) {
            fn bind(&self, site: &mut Site, var_name: &str) {
                (**self).bind(site, var_name);
            }
        }
    };
}

tuple_impl_site_bind!(0: A, 1: B);
tuple_impl_site_bind!(0: A, 1: B, 2: C);
tuple_impl_site_bind!(0: A, 1: B, 2: C, 3: D);
tuple_impl_site_bind!(0: A, 1: B, 2: C, 3: D, 4: E);
tuple_impl_site_bind!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F);
tuple_impl_site_bind!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G);
tuple_impl_site_bind!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H);
tuple_impl_site_bind!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I);
tuple_impl_site_bind!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J);
tuple_impl_site_bind!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K);
tuple_impl_site_bind!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L);
tuple_impl_site_bind!(0: A, 1: B, 2: C, 3: D, 4: E, 5: F, 6: G, 7: H, 8: I, 9: J, 10: K, 11: L, 12: M);
