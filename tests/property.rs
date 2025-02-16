use custom_vector_objones25::Vec;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_push_pop_sequence(operations in prop::collection::vec(0..100i32, 0..50)) {
        let mut vec = Vec::new();
        let mut reference = std::vec::Vec::new();
        
        // Apply operations
        for &value in &operations {
            vec.push(value);
            reference.push(value);
        }
        
        // Verify length
        prop_assert_eq!(vec.len(), reference.len());
        
        // Verify elements through pop
        while let Some(expected) = reference.pop() {
            prop_assert_eq!(vec.pop(), Some(expected));
        }
        
        // Both should be empty
        prop_assert_eq!(vec.pop(), None);
    }
    
    #[test]
    fn test_insert_remove_sequence(
        values in prop::collection::vec(0..100i32, 1..20),
        operations in prop::collection::vec((0..20usize, 0..100i32), 0..50)
    ) {
        let mut vec = Vec::new();
        let mut reference = std::vec::Vec::new();
        
        // Initial values
        for &value in &values {
            vec.push(value);
            reference.push(value);
        }
        
        // Apply operations
        for (index, value) in operations {
            if index % 2 == 0 && !reference.is_empty() {
                // Remove operation - ensure index is strictly less than length
                let index = index % reference.len();
                let removed = vec.remove(index);
                let expected = reference.remove(index);
                prop_assert_eq!(removed, expected);
            } else {
                // Insert operation - can insert at length (append)
                let index = index % (reference.len() + 1);
                vec.insert(index, value);
                reference.insert(index, value);
            }
            
            // Verify length and elements
            prop_assert_eq!(vec.len(), reference.len());
            for i in 0..reference.len() {
                prop_assert_eq!(vec[i], reference[i]);
            }
        }
    }
    
    #[test]
    fn test_extend_and_clear(
        chunks in prop::collection::vec(prop::collection::vec(0..100i32, 0..20), 0..10)
    ) {
        let mut vec = Vec::new();
        let mut reference = std::vec::Vec::new();
        
        for chunk in chunks {
            // Extend with chunk
            vec.extend(chunk.clone());
            reference.extend(chunk);
            
            // Verify length and elements
            prop_assert_eq!(vec.len(), reference.len());
            for i in 0..reference.len() {
                prop_assert_eq!(vec[i], reference[i]);
            }
            
            // Clear periodically
            if reference.len() > 10 {
                vec.clear();
                reference.clear();
                prop_assert!(vec.is_empty());
                prop_assert_eq!(vec.len(), 0);
            }
        }
    }
    
    #[test]
    fn test_iterator_properties(values in prop::collection::vec(0..100i32, 0..50)) {
        let mut vec = Vec::new();
        for &value in &values {
            vec.push(value);
        }
        
        // Test iterator equivalence
        let collected: std::vec::Vec<_> = vec.iter().copied().collect();
        prop_assert_eq!(collected.as_slice(), &values[..]);
        
        // Test double-ended iterator
        let rev_collected: std::vec::Vec<_> = vec.iter().rev().copied().collect();
        let mut expected_rev = values.clone();
        expected_rev.reverse();
        prop_assert_eq!(rev_collected.as_slice(), &expected_rev[..]);
        
        // Test mutable iterator
        for x in vec.iter_mut() {
            *x += 1;
        }
        
        // Verify modification
        for (i, &value) in values.iter().enumerate() {
            prop_assert_eq!(vec[i], value + 1);
        }
    }
    
    #[test]
    fn test_drain_properties(
        values in prop::collection::vec(0..100i32, 0..50),
        range_start in 0usize..50,
        range_len in 0usize..50
    ) {
        let mut vec = Vec::new();
        let mut reference = std::vec::Vec::new();
        
        // Setup vectors
        for &value in &values {
            vec.push(value);
            reference.push(value);
        }
        
        if !reference.is_empty() {
            let start = range_start % reference.len();
            let len = range_len % (reference.len() - start + 1);
            let end = start + len;
            
            // Collect drained elements
            let drained: std::vec::Vec<_> = vec.drain(start..end).collect();
            let reference_drained: std::vec::Vec<_> = reference.drain(start..end).collect();
            
            // Verify drained elements
            prop_assert_eq!(drained.as_slice(), reference_drained.as_slice());
            
            // Verify remaining elements
            prop_assert_eq!(vec.len(), reference.len());
            for i in 0..reference.len() {
                prop_assert_eq!(vec[i], reference[i]);
            }
        }
    }
}
