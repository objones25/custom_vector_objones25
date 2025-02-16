mod raw_vec;
mod iter;
use raw_vec::RawVec;
use std::ops::Index;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::IndexMut;
use std::slice;
use std::mem::ManuallyDrop;
use std::ptr;

pub struct Vec<T> {
    buf: RawVec<T>, // delegation to RawVec for memory management
    len: usize,
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        Vec {
            buf: RawVec::new(),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Vec {
            buf: RawVec::with_capacity(capacity),
            len: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(additional);
    }

    pub fn push(&mut self, item: T) {
        self.insert(self.len, item);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(self.buf.read_at(self.len))
        }
    }

    pub fn insert(&mut self, index: usize, item: T) {
        if index > self.len {
            panic!(
                "insertion index (is {}) should be <= len (is {})",
                index, self.len
            );
        }

        if self.capacity() == 0 {
            self.reserve(1);
        } else if self.len == self.capacity() {
            self.reserve(self.capacity());
        }

        self.buf.shift_right(index, self.len - index, 1);
        self.buf.write_at(index, item);
        self.len = self.len.checked_add(1)
            .expect("insert: length overflow");
    }


    pub fn remove(&mut self, index: usize) -> T {
        if index >= self.len {
            panic!(
                "removal index (is {}) should be < len (is {})",
                index, self.len
            )
        } else {
            let item: T = self.buf.read_at(index);
            let count = self.len - (index + 1); // Number of elements after the removed index
            self.buf.shift_left(index + 1, count, 1);
            self.len -= 1;
            item
        }
    }

    pub fn clear(&mut self) {
        while let Some(_) = self.pop() {}
    }
    
    pub fn truncate(&mut self, len: usize) {
        while self.len > len {
            self.pop();
        }
    }

    pub fn iter(&self) -> iter::Iter<'_, T> {
       iter::Iter::new(&self.buf, 0, self.len)
    }

    pub fn iter_mut(&mut self) -> iter::IterMut<'_, T> {
        // Create iterator that can modify elements
        iter::IterMut::new(&mut self.buf, 0, self.len)
    }

    pub fn drain(&mut self, range: std::ops::Range<usize>) -> iter::Drain<'_, T> {
        // Validate range
        assert!(range.start <= range.end);
        assert!(range.end <= self.len);
    
        // Store original length
        let orig_len = self.len;
        
        // Update vector length (elements will be removed)
        self.len -= range.end - range.start;
        
        // Create drain iterator
        iter::Drain::new(
            &mut self.buf,
            range.start,
            range.end,
            orig_len
        )
    }

    pub fn as_slice(&self) -> &[T] {
        self.deref()
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.deref_mut()
    }
}

impl<T> Index<usize> for Vec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("index (is {}) should be < len (is {})", index, self.len);
        }
        self.buf.get_ref(index)
    }
}

impl<T> IndexMut<usize> for Vec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len {
            panic!("index (is {}) should be < len (is {})", index, self.len);
        }

        self.buf.get_mut(index)
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.buf.ptr(), self.len) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.buf.ptr(), self.len) }
    }
}

impl<T: Clone> Clone for Vec<T> {
    fn clone(&self) -> Self {
        Vec {
            buf: self.buf.clone(),
            len: self.len,
        }
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = iter::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let vec = ManuallyDrop::new(self);
        
        iter::IntoIter::new(unsafe { ptr::read(&vec.buf) }, 0, vec.len)
    }
}

impl<T> FromIterator<T> for Vec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = Vec::new();
        vec.extend(iter);
        vec
    }
}

impl<T> Extend<T> for Vec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iter  = iter.into_iter();
        let (lower, upper) = iter.size_hint();
        self.reserve(upper.unwrap_or(lower));

        for item in iter {
            self.push(item);
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Vec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T> Default for Vec<T> {
    fn default() -> Self {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let vec: Vec<i32> = Vec::new();
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 0);
    }

    #[test]
    fn test_with_capacity() {
        let vec: Vec<i32> = Vec::with_capacity(10);
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 10);
    }

    #[test]
    fn test_push_pop() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.pop(), Some(3));
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.pop(), Some(2));
        assert_eq!(vec.pop(), Some(1));
        assert_eq!(vec.pop(), None);
    }

    #[test]
    fn test_insert_remove() {
        let mut vec = Vec::new();
        vec.insert(0, 1); // [1]
        vec.insert(1, 3); // [1, 3]
        vec.insert(1, 2); // [1, 2, 3]
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);
        assert_eq!(vec.remove(1), 2);
        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 3);
    }

    #[test]
    #[should_panic(expected = "insertion index (is 5) should be <= len (is 0)")]
    fn test_insert_out_of_bounds() {
        let mut vec: Vec<i32> = Vec::new();
        vec.insert(5, 10);
    }

    #[test]
    #[should_panic(expected = "removal index (is 5) should be < len (is 0)")]
    fn test_remove_out_of_bounds() {
        let mut vec: Vec<i32> = Vec::new();
        vec.remove(5);
    }

    #[test]
    fn test_clear_truncate() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.truncate(1);
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0], 1);
        vec.clear();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_indexing() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        assert_eq!(vec[0], 1);
        vec[1] = 3;
        assert_eq!(vec[1], 3);
    }

    #[test]
    #[should_panic(expected = "index (is 2) should be < len (is 2)")]
    fn test_index_out_of_bounds() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        let _ = vec[2];
    }

    #[test]
    fn test_clone() {
        let mut original = Vec::new();
        original.push(1);
        original.push(2);
        let cloned = original.clone();
        assert_eq!(original.len(), cloned.len());
        assert_eq!(original[0], cloned[0]);
        assert_eq!(original[1], cloned[1]);
    }

    #[test]
    fn test_extend() {
        let mut vec = Vec::new();
        vec.extend(vec![1, 2, 3]);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);
    }

    #[test]
    fn test_from_iterator() {
        let vec: Vec<i32> = (0..3).collect();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], 0);
        assert_eq!(vec[1], 1);
        assert_eq!(vec[2], 2);
    }

    #[test]
    fn test_debug_format() {
        let vec = vec![1, 2, 3];
        assert_eq!(format!("{:?}", vec), "[1, 2, 3]");
    }
}