use crate::RawVec; 
use std::ptr;

pub struct IntoIter<T> {
    buf: RawVec<T>,
    start: usize,
    end: usize,
}

pub struct Iter<'a, T> {
    buf: &'a RawVec<T>,
    start: usize,
    end: usize, // Tells Rust our iterator borrows T
}

pub struct IterMut<'a, T> {
    buf: &'a mut RawVec<T>,
    start: usize,
    end: usize,  // Shows we have mutable borrow
}

pub struct Drain<'a, T> {
    buf: &'a mut RawVec<T>,
    start: usize,
    end: usize,
    len: usize,
    original_start: usize,  // Store original start position
}

impl<T> IntoIter<T> {
    pub fn new(buf: RawVec<T>, start: usize, end: usize) -> Self {
        Self { buf, start, end }
    }
}

impl<'a, T> Iter<'a, T> {
    pub fn new(buf: &'a RawVec<T>, start: usize, end: usize) -> Self {
        Self { buf, start, end }
    }
}

impl<'a, T> IterMut<'a, T> {
    pub fn new(buf: &'a mut RawVec<T>, start: usize, end: usize) -> Self {
        Self { buf, start, end }
    }
}

impl<'a, T> Drain<'a, T> {
    pub fn new(buf: &'a mut RawVec<T>, start: usize, end: usize, len: usize) -> Self {
        Self { 
            buf, 
            start, 
            end, 
            len,
            original_start: start,  // Save original start position
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            let result: T = self.buf.read_at(self.start);
            self.start = self.start + 1;
            Some(result)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.end - self.start, Some(self.end - self.start))
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.start == self.end {
            None
        } else {
            let result: &T = self.buf.get_ref(self.start);
            self.start = self.start + 1;
            Some(result)
        }
    }   

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.end - self.start, Some(self.end - self.start))
    }
}   

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> {
        if self.start == self.end {
            None    
        } else {
            unsafe {
                let ptr: *mut T = self.buf.ptr().add(self.start);
                self.start += 1;
                Some(&mut *ptr)
            }
        }
    }           

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.end - self.start, Some(self.end - self.start))
    }
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        println!("Drain::next - start: {}, end: {}", self.start, self.end);
        if self.start == self.end {
            None
        } else {
            let result: T = self.buf.read_at(self.start);
            self.start += 1;
            Some(result)
        }
    } 

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.end - self.start, Some(self.end - self.start))
    }
}           

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            self.end -= 1;
            Some(self.buf.read_at(self.end))
        }
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<T> {
        println!("Drain::next_back - start: {}, end: {}", self.start, self.end);
        if self.start == self.end {
            None
        } else {
            self.end -= 1;
            Some(self.buf.read_at(self.end))
        }
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        // Drop any remaining elements in the drain range that weren't consumed
        while self.start < self.end {
            unsafe {
                ptr::drop_in_place(self.buf.ptr().add(self.start));
            }
            self.start += 1;
        }

        // Now shift any elements that were after the drain range
        let tail_remaining = self.len - self.end;
        if tail_remaining > 0 {
            self.buf.shift_left(
                self.end,      // Start of remaining elements
                tail_remaining,    // How many elements to move
                self.end - self.original_start  // How far to move them
            );
        }
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        if self.start == self.end {
            None
        } else {
            self.end -= 1;
            Some(self.buf.get_ref(self.end))
        }
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<&'a mut T> {
        if self.start == self.end {
            None
        } else {
            self.end -= 1;
            unsafe {
                let ptr = self.buf.ptr().add(self.end);
                Some(&mut *ptr)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RawVec;

    #[test]
    fn test_into_iter() {
        let mut raw = RawVec::with_capacity(3);
        raw.write_at(0, 1);
        raw.write_at(1, 2);
        raw.write_at(2, 3);
        let iter = IntoIter::new(raw, 0, 3);
        let collected: Vec<i32> = iter.collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_iter() {
        let mut raw = RawVec::with_capacity(3);
        raw.write_at(0, 1);
        raw.write_at(1, 2);
        raw.write_at(2, 3);
        let iter = Iter::new(&raw, 0, 3);
        let collected: Vec<&i32> = iter.collect();
        assert_eq!(collected, vec![&1, &2, &3]);
    }

    #[test]
    fn test_iter_mut() {
        let mut raw = RawVec::with_capacity(3);
        raw.write_at(0, 1);
        raw.write_at(1, 2);
        raw.write_at(2, 3);
        let iter = IterMut::new(&mut raw, 0, 3);
        let collected: Vec<i32> = iter.map(|x| *x).collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_drain() {
        let mut raw = RawVec::with_capacity(3);
        raw.write_at(0, 1);
        raw.write_at(1, 2);
        raw.write_at(2, 3);
        let drain = Drain::new(&mut raw, 0, 3, 3);
        let collected: Vec<i32> = drain.collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_into_iter_double_ended() {
        let mut raw = RawVec::with_capacity(3);
        raw.write_at(0, 1);
        raw.write_at(1, 2);
        raw.write_at(2, 3);
        let mut iter = IntoIter::new(raw, 0, 3);
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_iter_double_ended() {
        let mut raw = RawVec::with_capacity(3);
        raw.write_at(0, 1);
        raw.write_at(1, 2);
        raw.write_at(2, 3);
        let mut iter = Iter::new(&raw, 0, 3);
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next_back(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_iter_mut_double_ended() {
        let mut raw = RawVec::with_capacity(3);
        raw.write_at(0, 1);
        raw.write_at(1, 2);
        raw.write_at(2, 3);
        let mut iter = IterMut::new(&mut raw, 0, 3);
        assert_eq!(*iter.next().unwrap(), 1);
        assert_eq!(*iter.next_back().unwrap(), 3);
        assert_eq!(*iter.next().unwrap(), 2);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn test_drain_double_ended() {
        let mut raw = RawVec::with_capacity(3);
        raw.write_at(0, 1);
        raw.write_at(1, 2);
        raw.write_at(2, 3);
        let mut drain = Drain::new(&mut raw, 0, 3, 3);
        assert_eq!(drain.next(), Some(1));
        assert_eq!(drain.next_back(), Some(3));
        assert_eq!(drain.next(), Some(2));
        assert_eq!(drain.next(), None);
        assert_eq!(drain.next_back(), None);
    }

    #[test]
    fn test_size_hint() {
        // Test IntoIter
        let mut raw1 = RawVec::with_capacity(3);
        raw1.write_at(0, 1);
        raw1.write_at(1, 2);
        raw1.write_at(2, 3);
        let iter = IntoIter::new(raw1, 0, 3);
        assert_eq!(iter.size_hint(), (3, Some(3)));
        
        // Test immutable iterator
        let mut raw2 = RawVec::with_capacity(3);
        raw2.write_at(0, 1);
        raw2.write_at(1, 2);
        raw2.write_at(2, 3);
        let iter = Iter::new(&raw2, 0, 3);
        assert_eq!(iter.size_hint(), (3, Some(3)));
        
        // Test mutable iterator
        let iter = IterMut::new(&mut raw2, 0, 3);
        assert_eq!(iter.size_hint(), (3, Some(3)));
        drop(iter);  // Explicitly drop to release mutable borrow
        
        // Test drain
        let drain = Drain::new(&mut raw2, 0, 3, 3);
        assert_eq!(drain.size_hint(), (3, Some(3)));
    }
}