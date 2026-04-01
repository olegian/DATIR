/* This file is a part of the runtime library injected into the compiled project.
 * It defines the Tagged<T> type which ultimately represents a tuple (Id, T). All
 * tracked values are transformed into this tagged type to be able to uniquely 
 * identify where they are used. Id's are used within the various union find structure
 * (defined in ati.rs), to track interactions between values, and represent abstract
 * type sets.
*/

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
