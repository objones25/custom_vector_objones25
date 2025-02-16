use std::alloc::{self, Layout};
use std::ptr;

pub struct RawVec<T> {
    ptr: *mut T,      // Raw pointer to heap memory
    cap: usize,       // Total allocated capacity
}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        RawVec {
            ptr: ptr::null_mut(),
            cap: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        // Handle zero-sized types
        if std::mem::size_of::<T>() == 0 {
            return RawVec {
                ptr: std::ptr::NonNull::dangling().as_ptr(),
                cap: usize::MAX, // ZSTs can have "infinite" capacity
            };
        }

        // Add debug assertion for reasonable capacity
        debug_assert!(capacity <= isize::MAX as usize, 
            "capacity overflow: requested {} but max is {}", 
            capacity, isize::MAX);

        let layout: Layout = Layout::array::<T>(capacity)
            .expect("Failed to create layout for allocation");
        
        // Add debug assertion for layout size
        debug_assert!(layout.size() <= isize::MAX as usize,
            "allocation too large");

        let ptr: *mut T = unsafe { alloc::alloc(layout) as *mut T };
        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }
        RawVec { ptr, cap: capacity }
    }

    pub fn reserve(&mut self, additional: usize) {
        let new_cap = self.cap.checked_add(additional)
            .and_then(|cap| cap.checked_next_power_of_two())
            .expect("capacity overflow");


        if new_cap > self.cap {
            // Need to allocate new memory
            let new_layout = Layout::array::<T>(new_cap)
                .expect("layout overflow");
            unsafe {
                // Safely allocate new memory
                let new_ptr: *mut T = alloc::alloc(new_layout) as *mut T;
                if new_ptr.is_null() {
                    alloc::handle_alloc_error(new_layout);
                }
                // Copy old elements to new memory
                ptr::copy_nonoverlapping(self.ptr, new_ptr, self.cap);
                
                // Free old memory if we had any
                if self.cap > 0 {
                    let old_layout: Layout = Layout::array::<T>(self.cap)
                        .expect("Failed to create layout for deallocation");
                    alloc::dealloc(self.ptr as *mut u8, old_layout);
                }
                self.ptr = new_ptr;
                self.cap = new_cap;
            }
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn ptr(&self) -> *mut T {
        self.ptr
    }


    pub fn read_at(&self, index: usize) -> T {
        debug_assert!(index < self.cap, 
            "read_at: index {} out of bounds (cap: {})", 
            index, self.cap);
        debug_assert!(!self.ptr.is_null(), 
            "read_at: null pointer");
        
        unsafe {
            ptr::read(self.ptr.add(index))
        }
    }

    pub fn write_at(&mut self, index: usize, item: T) {
        debug_assert!(index < self.cap, 
            "write_at: index {} out of bounds (cap: {})", 
            index, self.cap);
        debug_assert!(!self.ptr.is_null(), 
            "write_at: null pointer");
        
        unsafe {
            ptr::write(self.ptr.add(index), item);
        }
    }

    pub fn get_ref(&self, index: usize) -> &T {
        debug_assert!(index < self.cap, 
            "get_ref: index {} out of bounds (cap: {})", 
            index, self.cap);
        debug_assert!(!self.ptr.is_null(), 
            "get_ref: null pointer");
        unsafe {
            &*self.ptr.add(index)
        }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        debug_assert!(index < self.cap, 
            "get_mut: index {} out of bounds (cap: {})", 
            index, self.cap);
        debug_assert!(!self.ptr.is_null(), 
            "get_mut: null pointer");
        unsafe {
            &mut *self.ptr.add(index)
        }
    }

    pub fn shift_right(&mut self, index: usize, count: usize, places: usize) {
        // Add debug assertions for bounds
        debug_assert!(index <= self.cap, 
            "shift_right: index out of bounds");
        debug_assert!(index.checked_add(count)
            .map_or(false, |end| end <= self.cap),
            "shift_right: count extends past capacity");
        debug_assert!(places.checked_add(index).is_some(), 
            "shift_right: places overflow");
        
        // Use checked arithmetic for new position
        let new_end = index.checked_add(places)
            .and_then(|x| x.checked_add(count))
            .expect("shift_right: position calculation overflow");
            
        if new_end > self.cap {
            self.reserve(new_end - self.cap);
        }
        
        unsafe {
            ptr::copy(
                self.ptr.add(index),
                self.ptr.add(index + places),
                count
            );
        }
    }

    pub fn shift_left(&mut self, index: usize, count: usize, places: usize) {
        // If count is 0, no need to do bounds checking
        if count == 0 {
            return;
        }
        
        // Add debug assertions for bounds
        debug_assert!(index >= places, 
            "shift_left: places larger than index");
        debug_assert!(index <= self.cap, 
            "shift_left: index out of bounds");
        debug_assert!(index.checked_add(count).map_or(false, |end| end <= self.cap),
            "shift_left: count extends past capacity");
        
        // Use checked arithmetic for new position
        let new_index = index.checked_sub(places)
            .expect("shift_left: index underflow");
        
        unsafe {
            ptr::copy(
                self.ptr.add(index),
                self.ptr.add(new_index),
                count
            );
        }
    }


}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        // Don't deallocate if:
        // 1. Capacity is 0 (never allocated)
        // 2. T is zero-sized (used a dangling pointer)
        if self.cap != 0 && std::mem::size_of::<T>() != 0 {
            unsafe {
                let layout: Layout = Layout::array::<T>(self.cap)
                    .expect("Failed to create layout for deallocation");
                alloc::dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
} 

impl<T: Clone> Clone for RawVec<T> {
    fn clone(&self) -> Self {
        let mut new_vec: RawVec<T> = Self::with_capacity(self.cap);
        for i in 0..self.cap {
            new_vec.write_at(i, self.read_at(i).clone());
        }
        new_vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_vec_new() {
        let raw: RawVec<i32> = RawVec::new();
        assert_eq!(raw.capacity(), 0);
        assert!(raw.ptr().is_null());
    }

    #[test]
    fn test_raw_vec_with_capacity() {
        let raw: RawVec<i32> = RawVec::with_capacity(10);
        assert_eq!(raw.capacity(), 10);
        assert!(!raw.ptr().is_null());
    }

    #[test]
    fn test_raw_vec_zero_sized() {
        let raw: RawVec<()> = RawVec::with_capacity(10);
        assert_eq!(raw.capacity(), usize::MAX);
        assert!(!raw.ptr().is_null());
    }

    #[test]
    fn test_raw_vec_reserve() {
        let mut raw: RawVec<i32> = RawVec::new();
        raw.reserve(5);
        assert!(raw.capacity() >= 5);
        let old_cap = raw.capacity();
        raw.reserve(10);
        assert!(raw.capacity() > old_cap);
    }

    #[test]
    fn test_raw_vec_read_write() {
        let mut raw: RawVec<i32> = RawVec::with_capacity(2);
        raw.write_at(0, 42);
        raw.write_at(1, 24);
        assert_eq!(raw.read_at(0), 42);
        assert_eq!(raw.read_at(1), 24);
    }

    #[test]
    fn test_raw_vec_get_ref() {
        let mut raw: RawVec<String> = RawVec::with_capacity(1);
        raw.write_at(0, String::from("test"));
        assert_eq!(raw.get_ref(0), "test");
    }

    #[test]
    fn test_raw_vec_get_mut() {
        let mut raw: RawVec<String> = RawVec::with_capacity(1);
        raw.write_at(0, String::from("test"));
        *raw.get_mut(0) = String::from("modified");
        assert_eq!(raw.read_at(0), "modified");
    }

    #[test]
    fn test_raw_vec_shift_right() {
        let mut raw: RawVec<i32> = RawVec::with_capacity(4);
        raw.write_at(0, 1);
        raw.write_at(1, 2);
        raw.shift_right(0, 2, 1);
        raw.write_at(0, 0);
        assert_eq!(raw.read_at(0), 0);
        assert_eq!(raw.read_at(1), 1);
        assert_eq!(raw.read_at(2), 2);
    }

    #[test]
    fn test_raw_vec_shift_left() {
        let mut raw: RawVec<i32> = RawVec::with_capacity(3);
        raw.write_at(0, 1);
        raw.write_at(1, 2);
        raw.write_at(2, 3);
        raw.shift_left(1, 2, 1);
        assert_eq!(raw.read_at(0), 2);
        assert_eq!(raw.read_at(1), 3);
    }

    #[test]
    fn test_raw_vec_clone() {
        let mut raw: RawVec<i32> = RawVec::with_capacity(2);
        raw.write_at(0, 1);
        raw.write_at(1, 2);
        let cloned = raw.clone();
        assert_eq!(cloned.read_at(0), 1);
        assert_eq!(cloned.read_at(1), 2);
        assert_eq!(cloned.capacity(), raw.capacity());
    }
}

