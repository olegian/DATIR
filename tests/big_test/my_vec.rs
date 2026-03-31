use std::alloc::{self, Layout};
use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct Vector<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T> Vector<T> {
    pub fn new() -> Self {
        // Start with no allocation
        Vector {
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0,
            _marker: PhantomData,
        }
    }

    pub fn push(&mut self, elem: T) {
        if self.len == self.cap {
            self.grow();
        }
        // SAFETY: `self.len` is less than `self.cap`, and the pointer is valid.
        // We also need to write the element without dropping the uninitialized memory.
        unsafe {
            std::ptr::write(self.ptr.as_ptr().add(self.len), elem);
        }
        self.len += 1;
    }

    fn grow(&mut self) {
        let new_cap = if self.cap == 0 { 1 } else { 2 * self.cap };
        // Ensure that T has a non-zero size for the Layout
        assert_ne!(std::mem::size_of::<T>(), 0, "Can't allocate for zero-sized types");

        let layout = Layout::array::<T>(new_cap).expect("Layout creation failed");

        // SAFETY: The new capacity is calculated to not overflow isize::MAX bytes
        let new_ptr = unsafe {
            if self.cap == 0 {
                alloc::alloc(layout)
            } else {
                // SAFETY: self.ptr is non-null and points to a valid allocation
                alloc::realloc(self.ptr.as_ptr() as *mut u8, Layout::array::<T>(self.cap).unwrap(), layout.size())
            }
        };

        // If allocation fails, we panic.
        self.ptr = NonNull::new(new_ptr as *mut T).expect("Memory reallocation failed");
        self.cap = new_cap;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            // SAFETY: `self.len` is now a valid index that contains an initialized element.
            Some(unsafe { std::ptr::read(self.ptr.as_ptr().add(self.len)) })
        }
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: self.ptr points to self.len initialized elements.
        unsafe {
            std::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }

    // Add other methods like `get`, `get_mut`, `&self[index]`, etc.
}

impl<T> Drop for Vector<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            // SAFETY: We must drop all initialized elements first.
            while let Some(_) = self.pop() {}
            let layout = Layout::array::<T>(self.cap).expect("Layout creation failed");
            // SAFETY: The allocation is valid and the layout is correct.
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}