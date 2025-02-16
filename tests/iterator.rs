use custom_vector_objones25::Vec;

#[test]
fn test_into_iter() {
    let mut vec = Vec::new();
    vec.extend(0..5);
    
    let mut iter = vec.into_iter();
    assert_eq!(iter.size_hint(), (5, Some(5)));
    
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next_back(), Some(4));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next_back(), Some(3));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn test_iter() {
    let mut vec = Vec::new();
    vec.extend(0..5);
    
    let mut iter = vec.iter();
    assert_eq!(iter.size_hint(), (5, Some(5)));
    
    assert_eq!(iter.next(), Some(&0));
    assert_eq!(iter.next_back(), Some(&4));
    assert_eq!(iter.next(), Some(&1));
    assert_eq!(iter.next_back(), Some(&3));
    assert_eq!(iter.next(), Some(&2));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
    
    // Original vec should still be valid
    assert_eq!(vec.len(), 5);
    assert_eq!(vec[0], 0);
}

#[test]
fn test_iter_mut() {
    let mut vec = Vec::new();
    vec.extend(0..5);
    
    let mut iter = vec.iter_mut();
    assert_eq!(iter.size_hint(), (5, Some(5)));
    
    // Modify through iterator
    while let Some(x) = iter.next() {
        *x *= 2;
    }
    
    // Verify modifications
    assert_eq!(vec[0], 0);
    assert_eq!(vec[1], 2);
    assert_eq!(vec[2], 4);
    assert_eq!(vec[3], 6);
    assert_eq!(vec[4], 8);
}

#[test]
fn test_drain() {
    let mut vec = Vec::new();
    vec.extend(0..5);
    
    // Drain middle elements
    {
        let drained: Vec<i32> = vec.drain(1..4).collect();
        assert_eq!(drained.len(), 3);
        assert_eq!(drained.as_slice(), &[1, 2, 3]);
    }
    
    // Verify remaining elements
    assert_eq!(vec.len(), 2);
    assert_eq!(vec[0], 0);
    assert_eq!(vec[1], 4);
    
    // Drain remaining elements
    {
        let drained: Vec<i32> = vec.drain(0..2).collect();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained.as_slice(), &[0, 4]);
    }
    
    // Vector should be empty
    assert!(vec.is_empty());
}

#[test]
fn test_iterator_collect() {
    let mut vec = Vec::new();
    vec.extend(0..5);
    
    // Collect into new vector
    let collected: Vec<i32> = vec.iter().copied().collect();
    assert_eq!(collected.len(), 5);
    for i in 0..5 {
        assert_eq!(collected[i], i as i32);
    }
    
    // Collect into String
    let string_vec = vec![String::from("Hello"), String::from("World")];
    let joined: String = string_vec.iter().fold(String::new(), |mut acc, s| {
        if !acc.is_empty() {
            acc.push(' ');
        }
        acc.push_str(s);
        acc
    });
    assert_eq!(joined, "Hello World");
}

#[test]
fn test_iterator_chain() {
    let mut vec1 = Vec::new();
    let mut vec2 = Vec::new();
    vec1.extend(0..3);
    vec2.extend(3..6);
    
    let chained: Vec<i32> = vec1.iter()
        .chain(vec2.iter())
        .copied()
        .collect();
    
    assert_eq!(chained.len(), 6);
    for i in 0..6 {
        assert_eq!(chained[i], i as i32);
    }
}

#[test]
fn test_iterator_zip() {
    let mut vec1 = Vec::new();
    let mut vec2 = Vec::new();
    vec1.extend(0..3);
    vec2.extend(3..6);
    
    let zipped: Vec<(i32, i32)> = vec1.iter()
        .zip(vec2.iter())
        .map(|(&a, &b)| (a, b))
        .collect();
    
    assert_eq!(zipped.len(), 3);
    assert_eq!(zipped[0], (0, 3));
    assert_eq!(zipped[1], (1, 4));
    assert_eq!(zipped[2], (2, 5));
}

#[test]
fn test_iterator_filter_map() {
    let mut vec = Vec::new();
    vec.extend(-2..3);
    
    let positive: Vec<i32> = vec.iter()
        .filter(|&&x| x > 0)
        .copied()
        .collect();
    
    assert_eq!(positive.len(), 2);
    assert_eq!(positive[0], 1);
    assert_eq!(positive[1], 2);
    
    let squared: Vec<i32> = vec.iter()
        .map(|&x| x * x)
        .collect();
    
    assert_eq!(squared.len(), 5);
    assert_eq!(squared[0], 4);  // (-2)^2
    assert_eq!(squared[1], 1);  // (-1)^2
    assert_eq!(squared[2], 0);  // 0^2
    assert_eq!(squared[3], 1);  // 1^2
    assert_eq!(squared[4], 4);  // 2^2
}
