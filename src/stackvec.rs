use std::{
    array,
    fmt::Debug,
    mem::MaybeUninit,
    ops::{Deref, DerefMut, Index, IndexMut},
};

pub struct StackVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> StackVec<T, N> {
    pub fn new() -> Self {
        StackVec {
            data: array::from_fn(|_| MaybeUninit::uninit()),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, val: T) {
        assert!(self.len < N);
        self.data[self.len].write(val);
        self.len += 1;
    }

    pub fn try_push(&mut self, val: T) -> Result<(), T> {
        if self.len >= N {
            return Err(val);
        }
        self.data[self.len].write(val);
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let cell = &self.data[self.len];
        Some(unsafe { cell.assume_init_read() })
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }
        //let val = std::mem::replace(&mut self.data[index], MaybeUninit::uninit());
        self.len -= 1;
        let mut last = std::mem::replace(&mut self.data[self.len], MaybeUninit::uninit());
        for i in (index..self.len).rev() {
            //self.data[i] = std::mem::replace(&mut self.data[i + 1], MaybeUninit::uninit());
            last = std::mem::replace(&mut self.data[i], last);
        }
        Some(unsafe { last.assume_init_read() })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.as_ref().iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.as_mut().iter_mut()
    }
}

impl<T, const N: usize> AsRef<[T]> for StackVec<T, N> {
    fn as_ref(&self) -> &[T] {
        let slice = &self.data[..self.len];
        unsafe { std::mem::transmute(slice) }
    }
}

impl<T, const N: usize> AsMut<[T]> for StackVec<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        let slice = &mut self.data[..self.len];
        unsafe { std::mem::transmute(slice) }
    }
}

impl<T, const N: usize> Deref for StackVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_ref()
    }
}

impl<T, const N: usize> DerefMut for StackVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T, const N: usize> Default for StackVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Debug, const N: usize> Debug for StackVec<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<T, const N: usize> Drop for StackVec<T, N> {
    fn drop(&mut self) {
        for i in 0..self.len {
            unsafe {
                self.data[i].assume_init_read();
            }
        }
    }
}

impl<T: Clone, const N: usize> Clone for StackVec<T, N> {
    fn clone(&self) -> Self {
        let mut cloned = Self::new();
        cloned.len = self.len;
        self.iter().enumerate().for_each(|(i, val)| {
            cloned.data[i].write(val.clone());
        });
        cloned
    }
}

impl<T, const N: usize> Index<usize> for StackVec<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        assert!(index < self.len);
        unsafe { self.data[index].assume_init_ref() }
    }
}

impl<T, const N: usize> IndexMut<usize> for StackVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len);
        unsafe { self.data[index].assume_init_mut() }
    }
}
