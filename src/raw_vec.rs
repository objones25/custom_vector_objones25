use std::alloc::{self, Layout};
use std::ptr;

pub struct RawVec<T> {
    ptr: *mut T,      // Raw pointer to heap memory
    cap: usize,       // Total allocated capacity
}

impl<T> RawVec<T> {
    // Growth strategy constants
    const MIN_NON_ZERO_CAP: usize = 8;  // Minimum non-zero capacity
    const SHRINK_THRESHOLD: f64 = 0.25;  // Shrink when waste > 25%
    const MAX_EXCESS_CAPACITY: usize = 1024 * 1024;  // 1MB worth of elements
    
    // Platform-specific constants
    #[cfg(target_pointer_width = "64")]
    const MAX_CAPACITY: usize = 1 << 48;  // 256 TB on 64-bit

    #[cfg(target_pointer_width = "32")]
    const MAX_CAPACITY: usize = 1 << 30;  // 1 GB on 32-bit

    fn calculate_growth(&self, required_cap: usize) -> usize {
        let elem_size = std::mem::size_of::<T>();
        
        // Calculate minimum growth based on element size
        let min_growth = if elem_size > 1024 {
            // Large elements (>1KB): grow by 25%
            let growth = self.cap.saturating_add(self.cap / 4);
            // Never round up for large types to ensure we stay under 50%
            std::cmp::min(growth, required_cap)
        } else if elem_size > 128 {
            // Medium elements (>128B): grow by 50%
            let growth = self.cap.saturating_add(self.cap / 2);
            // Never round up for medium types to ensure we stay under 2x
            std::cmp::min(growth, required_cap)
        } else {
            // Small elements: grow by 100%
            self.cap.saturating_mul(2)
        };

        // For small types only, use the larger of minimum growth or required capacity
        let mut new_cap = if elem_size <= 128 {
            std::cmp::max(min_growth, required_cap)
        } else {
            min_growth
        };

        // Only round to power of 2 for small types
        if elem_size <= 128 {
            let next_pow2 = new_cap.next_power_of_two();
            let waste = next_pow2.saturating_sub(new_cap);
            if next_pow2 <= new_cap.saturating_add(new_cap / 8) && // Within 12.5%
               waste * elem_size <= 16 * 1024 {  // Waste <= 16KB
                new_cap = next_pow2;
            }
        }

        // Ensure we don't exceed platform capacity
        std::cmp::min(new_cap, Self::MAX_CAPACITY)
    }

    pub fn new() -> Self {
        // For zero-sized types, use dangling pointer and cap 0
        if std::mem::size_of::<T>() == 0 {
            RawVec {
                ptr: std::ptr::NonNull::dangling().as_ptr(),
                cap: 0,
            }
        } else {
            RawVec {
                ptr: ptr::null_mut(),
                cap: 0,
            }
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        // Handle zero-sized types
        if std::mem::size_of::<T>() == 0 {
            return RawVec {
                ptr: std::ptr::NonNull::dangling().as_ptr(),
                cap: usize::MAX, // ZSTs always have maximum capacity
            };
        }

        // Only use MIN_NON_ZERO_CAP for zero capacity
        let capacity = if capacity == 0 {
            Self::MIN_NON_ZERO_CAP
        } else {
            capacity
        };

        // Add debug assertion for reasonable capacity
        debug_assert!(capacity <= isize::MAX as usize, 
            "capacity overflow: requested {} but max is {}", 
            capacity, isize::MAX);

        let layout = Layout::array::<T>(capacity)
            .expect("Failed to create layout for allocation");
        
        // Add debug assertion for layout size
        debug_assert!(layout.size() <= isize::MAX as usize,
            "allocation too large");

        let ptr = unsafe { alloc::alloc(layout) as *mut T };
        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }
        RawVec { ptr, cap: capacity }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn reserve(&mut self, additional: usize) {
        // Handle zero-sized types
        if std::mem::size_of::<T>() == 0 {
            self.cap = usize::MAX;
            return;
        }

        // Calculate required capacity with overflow checking
        let required_cap = self.cap.checked_add(additional)
            .unwrap_or(Self::MAX_CAPACITY);
        
        // Calculate maximum possible elements based on size
        let max_elements = std::cmp::min(
            isize::MAX as usize / std::mem::size_of::<T>().max(1),
            Self::MAX_CAPACITY
        );
        
        // Cap at maximum possible elements
        let capped_required = std::cmp::min(required_cap, max_elements);
        
        // If we don't need to grow, return early
        if capped_required <= self.cap {
            return;
        }

        // If we're starting from zero, use MIN_NON_ZERO_CAP
        if self.cap == 0 {
            let new_cap = std::cmp::max(capped_required, Self::MIN_NON_ZERO_CAP);
            self.grow_to(new_cap.next_power_of_two());
            return;
        }

        // Calculate growth with capped capacity
        let new_cap = self.calculate_growth(capped_required);
        self.grow_to(new_cap);
    }

    fn grow_to(&mut self, new_cap: usize) {
        debug_assert!(new_cap > self.cap);
        debug_assert!(new_cap <= Self::MAX_CAPACITY, "Attempted to grow beyond MAX_CAPACITY");
        
        // Handle zero-sized types
        if std::mem::size_of::<T>() == 0 {
            self.cap = new_cap;
            return;
        }

        // Check if allocation size is reasonable
        let layout = Layout::array::<T>(new_cap);
        if layout.is_err() || layout.as_ref().map_or(true, |l| l.size() > isize::MAX as usize) {
            // If allocation would be too large, cap at current reasonable maximum
            let max_elements = std::cmp::min(
                isize::MAX as usize / std::mem::size_of::<T>(),
                Self::MAX_CAPACITY
            );
            if self.cap >= max_elements {
                return;
            }
            self.grow_to(max_elements);
            return;
        }
        
        let new_layout = layout.unwrap();
        
        unsafe {
            let new_ptr = if self.cap == 0 {
                // Fast path: no need to copy when growing from empty
                alloc::alloc(new_layout) as *mut T
            } else {
                // Attempt to use realloc for potential performance gain
                let old_layout = Layout::array::<T>(self.cap)
                    .expect("old layout overflow");
                let new_ptr = alloc::realloc(
                    self.ptr as *mut u8,
                    old_layout,
                    new_layout.size()
                ) as *mut T;
                
                if !new_ptr.is_null() {
                    // Realloc succeeded
                    new_ptr
                } else {
                    // Fallback to alloc + copy + dealloc
                    let new_ptr = alloc::alloc(new_layout) as *mut T;
                    if !new_ptr.is_null() {
                        ptr::copy_nonoverlapping(self.ptr, new_ptr, self.cap);
                        alloc::dealloc(self.ptr as *mut u8, old_layout);
                    }
                    new_ptr
                }
            };

            if new_ptr.is_null() {
                alloc::handle_alloc_error(new_layout);
            }
            
            self.ptr = new_ptr;
            self.cap = new_cap;
        }
    }

    fn should_shrink(&self, len: usize) -> bool {
        let elem_size = std::mem::size_of::<T>();
        
        // Don't shrink zero-sized types
        if elem_size == 0 {
            return false;
        }
        
        // Calculate waste
        let waste = self.cap - len;
        let waste_bytes = waste * elem_size;
        
        // Calculate usage ratio
        let usage_ratio = (len as f64) / (self.cap as f64);
        let waste_ratio = 1.0 - usage_ratio;
        
        // Shrink if either:
        // 1. We're wasting significant memory (>25% AND >1MB)
        // 2. OR we're using less than 25% of capacity (regardless of size)
        (waste_bytes > Self::MAX_EXCESS_CAPACITY && waste_ratio > Self::SHRINK_THRESHOLD) ||
        (usage_ratio < Self::SHRINK_THRESHOLD)
    }

    pub fn shrink_to_fit(&mut self, len: usize) {
        // Handle zero-sized types
        if std::mem::size_of::<T>() == 0 {
            self.cap = usize::MAX;
            return;
        }

        // Don't shrink if length is close to capacity
        if len >= self.cap - self.cap / 4 {
            return;
        }

        // Only shrink if we should according to our criteria
        if self.should_shrink(len) {
            // Calculate new capacity: round up to next power of 2 above len
            let mut new_cap = len.next_power_of_two();
            
            // But don't go below MIN_NON_ZERO_CAP
            new_cap = std::cmp::max(new_cap, Self::MIN_NON_ZERO_CAP);
            
            // Only proceed if we're actually reducing capacity
            if new_cap < self.cap {
                unsafe {
                    let new_layout = Layout::array::<T>(new_cap)
                        .expect("layout overflow");
                    let new_ptr = alloc::alloc(new_layout) as *mut T;
                    
                    if !new_ptr.is_null() {
                        ptr::copy_nonoverlapping(self.ptr, new_ptr, len);
                        let old_layout = Layout::array::<T>(self.cap)
                            .expect("old layout overflow");
                        alloc::dealloc(self.ptr as *mut u8, old_layout);
                        self.ptr = new_ptr;
                        self.cap = new_cap;
                    }
                }
            }
        }
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
        assert!(raw.capacity() >= 10);
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

    #[test]
    fn test_growth_properties() {
        let mut vec = RawVec::<i32>::new();
        
        // Test initial allocation is power of 2
        vec.reserve(1);
        assert!(vec.capacity().is_power_of_two());
        assert!(vec.capacity() >= RawVec::<i32>::MIN_NON_ZERO_CAP);

        // Test growth maintains minimum capacity
        let mut last_cap = vec.capacity();
        for i in 1..=10 {
            vec.reserve(i * 100);
            let new_cap = vec.capacity();
            assert!(new_cap >= last_cap, "Capacity should never decrease on reserve");
            assert!(new_cap >= i * 100, "Capacity should meet required size");
            last_cap = new_cap;
        }
    }

    #[test]
    fn test_shrink_properties() {
        let mut vec = RawVec::<i32>::with_capacity(1024);
        let original_cap = vec.capacity();
        
        // Shrinking to 1/8th of capacity should trigger resize
        vec.shrink_to_fit(original_cap / 8);
        assert!(vec.capacity() < original_cap);
        assert!(vec.capacity() >= original_cap / 8);
        assert!(vec.capacity().is_power_of_two());
        
        // Shrinking to 90% of current shouldn't change capacity
        let current_cap = vec.capacity();
        vec.shrink_to_fit((current_cap * 9) / 10);
        assert_eq!(vec.capacity(), current_cap);
    }

    #[test]
    fn test_zero_sized_type_handling() {
        let mut vec = RawVec::<()>::new();
        assert_eq!(vec.capacity(), 0);
        
        vec.reserve(100);
        assert_eq!(vec.capacity(), usize::MAX);
        
        vec.shrink_to_fit(50);
        assert_eq!(vec.capacity(), usize::MAX);
    }

    #[test]
    fn test_type_size_aware_growth() {
        // Test small type (i32 = 4 bytes)
        let mut small_vec = RawVec::<i32>::new();
        small_vec.reserve(10);
        let small_cap1 = small_vec.capacity();
        small_vec.reserve(small_cap1 + 1);
        let small_cap2 = small_vec.capacity();
        // For small types, should grow by ~100%
        assert!(small_cap2 >= small_cap1 * 2);

        // Test medium type (512 bytes)
        #[derive(Clone)]
        #[allow(dead_code)]
        struct MediumType([u8; 512]);
        let mut medium_vec = RawVec::<MediumType>::new();
        medium_vec.reserve(10);
        let med_cap1 = medium_vec.capacity();
        medium_vec.reserve(med_cap1 + 1);
        let med_cap2 = medium_vec.capacity();
        // For medium types, should grow by ~50%
        assert!(med_cap2 >= med_cap1 + med_cap1 / 2);
        assert!(med_cap2 < med_cap1 * 2);

        // Test large type (2KB)
        #[derive(Clone)]
        #[allow(dead_code)]
        struct LargeType([u8; 2048]);
        let mut large_vec = RawVec::<LargeType>::new();
        large_vec.reserve(10);
        let large_cap1 = large_vec.capacity();
        large_vec.reserve(large_cap1 + 1);
        let large_cap2 = large_vec.capacity();
        // For large types, should grow by ~25%
        assert!(large_cap2 >= large_cap1 + large_cap1 / 4);
        assert!(large_cap2 < large_cap1 + large_cap1 / 2);
    }

    #[test]
    fn test_memory_waste_control() {
        // Test that we don't round to power of 2 when it would waste too much memory
        let mut vec = RawVec::<[u8; 1024]>::new();
        vec.reserve(1000);  // Just under 1024
        let cap = vec.capacity();
        // Should not round up to 2048 as it would waste >16KB
        assert!(cap < 2048);
        
        // Test with small type where rounding is acceptable
        let mut small_vec = RawVec::<u8>::new();
        small_vec.reserve(1000);
        let small_cap = small_vec.capacity();
        // Should round up to 1024 as waste is minimal
        assert_eq!(small_cap, 1024);
    }

    #[test]
    fn test_shrink_memory_threshold() {
        // Create a vector with 1MB worth of elements
        #[allow(dead_code)]
        struct TestType([u8; 1024]);
        
        let mut vec = RawVec::<TestType>::with_capacity(2048);  // 2MB total
        
        // Shrinking to 1/4 capacity (512KB waste) should trigger shrink
        vec.shrink_to_fit(512);  // Now using 512KB
        assert!(vec.capacity() < 2048);
        
        // Shrinking with less waste shouldn't trigger
        let cap = vec.capacity();
        vec.shrink_to_fit(cap - 1);
        assert_eq!(vec.capacity(), cap);
    }

    #[test]
    fn test_platform_capacity_limits() {
        let mut vec = RawVec::<u8>::new();
        
        // Start with a small size
        vec.reserve(64);
        let initial_cap = vec.capacity();
        assert!(initial_cap >= 64);
        
        // Test moderate growth
        let test_sizes = [
            initial_cap * 2,            // Double
            initial_cap * 4,            // Quadruple
            initial_cap * 8,            // 8x
            initial_cap * 16,           // 16x
            1024 * 1024,               // 1MB worth of elements
        ];
        
        for &size in &test_sizes {
            vec.reserve(size);
            assert!(vec.capacity() <= RawVec::<u8>::MAX_CAPACITY);
            assert!(vec.capacity() >= initial_cap);
            // Ensure we can still use the allocated memory
            unsafe {
                std::ptr::write_bytes(vec.ptr(), 0xAA, vec.capacity());
            }
        }
        
        // Test with a medium-sized type
        let mut medium_vec = RawVec::<[u8; 64]>::new();
        medium_vec.reserve(64);  // Try to allocate 4KB
        assert!(medium_vec.capacity() <= RawVec::<[u8; 64]>::MAX_CAPACITY);
        
        // Ensure we can still grow from non-zero capacity
        let cap_before = medium_vec.capacity();
        medium_vec.reserve(cap_before * 2);
        assert!(medium_vec.capacity() >= cap_before);
        assert!(medium_vec.capacity() <= RawVec::<[u8; 64]>::MAX_CAPACITY);
    }
}

