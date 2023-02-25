pub struct StaticVec<T: Copy, const N: usize> {
    data: [T; N],
    len: usize,
}

impl<T: Copy, const N: usize> StaticVec<T, N> {
    pub fn new(default: T) -> Self {
        Self {
            data: [default; N],
            len: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        self.data[self.len] = item;
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data[..self.len]
    }
}
