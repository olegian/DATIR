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

/// helpful for debugging purposes, allowing printing of tagged values.
impl<T> std::fmt::Display for Tagged<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
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
