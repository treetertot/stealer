pub struct UnsafeVecBuilder<T> {
    target: *mut Vec<Option<T>>,
}
unsafe impl<T> Send for UnsafeVecBuilder<T> where T: Send {}
unsafe impl<T> Sync for UnsafeVecBuilder<T> where T: Send {}
impl<T> UnsafeVecBuilder<T> {
    pub unsafe fn insert(&mut self, index: usize, item: T) {
        (*self.target)[index] = Some(item);
    }
}
impl<T> Clone for UnsafeVecBuilder<T> {
    fn clone(&self) -> Self {
        UnsafeVecBuilder{ target: self.target }
    }
}

pub struct SharedVec<T> (Vec<Option<T>>);
impl<T> SharedVec<T> {
    pub fn new(cap: usize) -> Self {
        let mut base = Vec::with_capacity(cap);
        for _ in 0..cap {
            base.push(None);
        }
        SharedVec(base)
    }
    pub fn builder(&mut self) -> UnsafeVecBuilder<T> {
        let ptr = (&mut self.0) as *mut Vec<Option<T>>;
        UnsafeVecBuilder {
            target: ptr
        }
    }
    pub fn collapse(self) -> Option<Vec<T>> {
        let mut out = Vec::with_capacity(self.0.len());
        for item in self.0 {
            out.push(item?);
        }
        Some(out)
    }
}

