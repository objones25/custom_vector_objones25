# Vector Implementation Baseline Performance Metrics

## Growth Pattern Operations

### Push Consecutive (No Pre-allocation)
- 100 elements: 320.81 ± 2.54 ns
- 1,000 elements: 1.16 ± 0.005 µs
- 10,000 elements: 11.24 ± 0.50 µs
- 100,000 elements: 165.99 ± 2.37 µs

### Reserve Then Push (With Pre-allocation)
- 100 elements: 83.93 ± 0.57 ns
- 1,000 elements: 488.40 ± 6.45 ns
- 10,000 elements: 5.16 ± 0.21 µs
- 100,000 elements: 70.54 ± 1.88 µs

### Extend From Iterator
- 100 elements: 83.36 ± 0.13 ns
- 1,000 elements: 691.65 ± 3.59 ns
- 10,000 elements: 6.79 ± 0.06 µs
- 100,000 elements: 67.90 ± 0.37 µs

## Common Operations (by vector size)

### Size: 1,000 elements
- Insert Start: 289.44 ± 11.61 ns
- Remove Start: 152.90 ± 1.57 ns
- Insert Middle: 253.59 ± 11.51 ns
- Remove Middle: 79.38 ± 2.01 ns
- Insert End: 220.11 ± 11.97 ns
- Remove End: 2.55 ± 0.03 ns
- Extend: 321.38 ± 7.71 ns
- Iterate: 275.09 ± 0.97 ns

### Size: 10,000 elements
- Insert Start: 936.97 ± 9.64 ns
- Remove Start: 1.49 ± 0.005 µs
- Insert Middle: 595.71 ± 10.56 ns
- Remove Middle: 749.58 ± 2.77 ns
- Insert End: 226.65 ± 11.45 ns
- Remove End: 2.58 ± 0.03 ns
- Extend: 2.25 ± 0.48 µs
- Iterate: 2.52 ± 0.02 µs

### Size: 100,000 elements
- Insert Start: 12.07 ± 0.04 µs
- Remove Start: 22.41 ± 0.04 µs
- Insert Middle: 6.24 ± 0.03 µs
- Remove Middle: 10.69 ± 0.04 µs
- Insert End: 230.85 ± 11.67 ns
- Remove End: 2.51 ± 0.03 ns
- Extend: 13.08 ± 0.45 µs
- Iterate: 24.95 ± 0.14 µs

## Memory Pattern Operations

### Grow-Shrink Pattern
- 1,024 elements: 1.76 ± 0.01 µs
- 4,096 elements: 6.11 ± 0.05 µs
- 16,384 elements: 27.68 ± 0.79 µs

### Clear and Reuse Pattern
- 1,024 elements: 1.01 ± 0.005 µs
- 4,096 elements: 4.02 ± 0.02 µs
- 16,384 elements: 17.96 ± 0.54 µs

## Key Observations

1. Growth Efficiency:
   - Pre-allocated pushes are ~3.8x faster than non-pre-allocated for 100 elements
   - This efficiency improves to ~2.4x for 100,000 elements
   - Extend from iterator performs similarly to pre-allocated pushes

2. Operation Costs:
   - End operations are consistently fastest (O(1) complexity)
   - Start operations are most expensive (O(n) shifting required)
   - Middle operations scale better than start operations for large sizes

3. Memory Pattern Efficiency:
   - Clear-and-reuse is ~1.7x faster than grow-shrink
   - This efficiency ratio remains relatively constant across sizes

4. Notable Performance Characteristics:
   - Sub-microsecond performance for small vectors (<1000 elements)
   - Linear scaling for most operations
   - Excellent end-operation performance
   - Efficient iterator implementation

## Areas for Potential Optimization

1. Growth Strategy:
   - Could improve non-pre-allocated push performance
   - Potential for better growth factor tuning

2. Memory Management:
   - Shrink-to-fit opportunities
   - Reallocation strategy refinement

3. Operation Optimizations:
   - Start/Middle operation improvements
   - Bulk operation efficiency
   - Memory movement optimization

4. Iterator Performance:
   - Already good, but could potentially improve for large collections 