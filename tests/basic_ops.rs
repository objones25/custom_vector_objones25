use custom_vector_objones25::Vec;

#[test]
fn test_vec_basic_operations() {
    let mut vec = Vec::new();
    
    // Test push and length
    vec.push(1);
    vec.push(2);
    vec.push(3);
    assert_eq!(vec.len(), 3);
    
    // Test indexing
    assert_eq!(vec[0], 1);
    assert_eq!(vec[1], 2);
    assert_eq!(vec[2], 3);
    
    // Test pop
    assert_eq!(vec.pop(), Some(3));
    assert_eq!(vec.len(), 2);
    
    // Test insert
    vec.insert(1, 4);
    assert_eq!(vec[0], 1);
    assert_eq!(vec[1], 4);
    assert_eq!(vec[2], 2);
    
    // Test remove
    assert_eq!(vec.remove(1), 4);
    assert_eq!(vec.len(), 2);
    assert_eq!(vec[0], 1);
    assert_eq!(vec[1], 2);
}

#[test]
fn test_vec_capacity_management() {
    let mut vec = Vec::with_capacity(2);
    assert!(vec.capacity() >= 2);
    
    // Fill to capacity
    vec.push(1);
    vec.push(2);
    let initial_capacity = vec.capacity();
    
    // Force growth
    vec.push(3);
    assert!(vec.capacity() > initial_capacity);
    
    // Test reserve
    let old_capacity = vec.capacity();
    vec.reserve(10);
    assert!(vec.capacity() >= old_capacity + 10);
}

#[test]
fn test_vec_clear_and_truncate() {
    let mut vec = Vec::new();
    for i in 0..5 {
        vec.push(i);
    }
    
    // Test truncate
    vec.truncate(3);
    assert_eq!(vec.len(), 3);
    assert_eq!(vec[0], 0);
    assert_eq!(vec[1], 1);
    assert_eq!(vec[2], 2);
    
    // Test clear
    vec.clear();
    assert_eq!(vec.len(), 0);
    assert!(vec.is_empty());
}

#[test]
fn test_vec_iteration() {
    let mut vec = Vec::new();
    for i in 0..3 {
        vec.push(i);
    }
    
    // Test iterator
    let mut sum = 0;
    for &x in vec.iter() {
        sum += x;
    }
    assert_eq!(sum, 3);
    
    // Test mutable iterator
    for x in vec.iter_mut() {
        *x += 1;
    }
    assert_eq!(vec[0], 1);
    assert_eq!(vec[1], 2);
    assert_eq!(vec[2], 3);
    
    // Test drain
    let drained: Vec<i32> = vec.drain(1..3).collect();
    assert_eq!(drained.as_slice(), &[2, 3]);
    assert_eq!(vec.len(), 1);
    assert_eq!(vec[0], 1);
}

#[test]
fn test_vec_clone_and_extend() {
    let mut original = Vec::new();
    original.push(1);
    original.push(2);
    
    // Test clone
    let cloned = original.clone();
    assert_eq!(cloned.len(), original.len());
    assert_eq!(cloned[0], original[0]);
    assert_eq!(cloned[1], original[1]);
    
    // Test extend
    original.extend(vec![3, 4, 5]);
    assert_eq!(original.len(), 5);
    assert_eq!(original[2], 3);
    assert_eq!(original[3], 4);
    assert_eq!(original[4], 5);
}

#[test]
fn test_vec_from_iterator() {
    // Test collecting into Vec
    let vec: Vec<i32> = (0..5).collect();
    assert_eq!(vec.len(), 5);
    for i in 0..5 {
        assert_eq!(vec[i], i as i32);
    }
    
    // Test into_iter
    let sum: i32 = vec.into_iter().sum();
    assert_eq!(sum, 10);
}
