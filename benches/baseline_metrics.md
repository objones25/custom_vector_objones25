# Vector Implementation Baseline Performance Metrics

## Growth Pattern Operations

### Push Consecutive (No Pre-allocation)
- 100 elements: 184.19 ± 2.59 ns (-42.53% improvement)
- 1,000 elements: 767.30 ± 3.07 ns (-33.64% improvement)
- 10,000 elements: 14.31 ± 0.44 µs (+32.82% regression)
- 100,000 elements: 111.36 ± 1.85 µs (-33.47% improvement)

### Reserve Then Push (With Pre-allocation)
- 100 elements: 66.57 ± 0.43 ns (-21.18% improvement)
- 1,000 elements: 336.38 ± 1.30 ns (-30.75% improvement)
- 10,000 elements: 3.50 ± 0.06 µs (-27.49% improvement)
- 100,000 elements: 79.49 ± 0.20 µs (+6.56% regression)

### Extend From Iterator
- 100 elements: 65.34 ± 0.42 ns (-20.85% improvement)
- 1,000 elements: 477.35 ± 1.09 ns (-30.91% improvement)
- 10,000 elements: 4.64 ± 0.03 µs (-31.37% improvement)
- 100,000 elements: 69.95 ± 3.85 µs (+7.74% regression)

## Common Operations (by vector size)

### Size: 1,000 elements
- Insert Start: 289.66 ± 11.17 ns (no significant change)
- Remove Start: 145.38 ± 0.66 ns (-4.44% improvement)
- Insert Middle: 254.81 ± 11.76 ns (no significant change)
- Remove Middle: 73.03 ± 0.16 ns (-8.83% improvement)
- Insert End: 221.22 ± 12.02 ns (no significant change)
- Remove End: 2.90 ± 0.01 ns (+14.43% regression)
- Extend: 231.41 ± 2.05 ns (-27.94% improvement)
- Iterate: 268.91 ± 0.63 ns (-2.35% improvement)

### Size: 10,000 elements
- Insert Start: 928.32 ± 10.32 ns (no significant change)
- Remove Start: 1.49 ± 0.004 µs (no significant change)
- Insert Middle: 580.75 ± 11.82 ns (no significant change)
- Remove Middle: 747.30 ± 3.01 ns (no significant change)
- Insert End: 229.91 ± 12.15 ns (no significant change)
- Remove End: 2.92 ± 0.01 ns (+11.74% regression)
- Extend: 6.06 ± 0.35 µs (+158.09% regression)
- Iterate: 2.51 ± 0.03 µs (no significant change)

### Size: 100,000 elements
- Insert Start: 11.96 ± 0.04 µs (no significant change)
- Remove Start: 22.52 ± 0.32 µs (no significant change)
- Insert Middle: 6.15 ± 0.02 µs (-1.29% improvement)
- Remove Middle: 10.58 ± 0.04 µs (-0.86% improvement)
- Insert End: 224.21 ± 11.64 ns (-14.14% improvement)
- Remove End: 2.88 ± 0.03 ns (+14.32% regression)
- Extend: 11.50 ± 0.24 µs (-11.30% improvement)
- Iterate: 24.57 ± 0.08 µs (-2.81% improvement)

## Memory Pattern Operations

### Grow-Shrink Pattern
- 1,024 elements: 1.25 ± 0.01 µs (-29.39% improvement)
- 4,096 elements: 4.11 ± 0.01 µs (-32.09% improvement)
- 16,384 elements: 32.87 ± 0.58 µs (+17.16% regression)

### Clear and Reuse Pattern
- 1,024 elements: 695.20 ± 2.20 ns (-31.88% improvement)
- 4,096 elements: 2.80 ± 0.01 µs (-30.82% improvement)
- 16,384 elements: 14.45 ± 0.03 µs (-19.82% improvement)

## Impact Analysis of Type-Size Aware Growth Strategy

### Key Improvements

1. Small Vector Operations (≤1000 elements):
   - Significant improvements across all growth patterns
   - Push operations without pre-allocation improved by 33-42%
   - Pre-allocated operations improved by 21-30%
   - Memory efficiency gains reflected in better extend performance

2. Memory Management:
   - Clear-and-reuse pattern shows consistent improvements (19-31%)
   - Small/medium grow-shrink operations improved by 29-32%
   - Better memory utilization due to type-size aware growth

3. Common Operations:
   - Notable improvements in extend operations for small vectors
   - Middle and end operations show modest improvements
   - Iterator performance improved slightly across all sizes

### Areas of Regression

1. Large Vector Operations (>10K elements):
   - Push consecutive performance regression at 10K elements (+32.82%)
   - Reserve-then-push shows slight regression at 100K elements (+6.56%)
   - Extend operation regression at 10K elements (+158.09%)

2. Memory Operations:
   - Grow-shrink pattern shows regression for large sizes (16K elements: +17.16%)
   - Remove-end operation consistently shows small regression (+11-14%)

3. Trade-offs:
   - The type-size aware strategy optimizes for memory efficiency at the cost of some large-vector performance
   - Small vector operations benefit significantly from the new approach
   - Memory reuse patterns show better performance than grow-shrink patterns

## Optimization Recommendations

1. Growth Strategy Refinement:
   - Consider hybrid approach for vectors >10K elements
   - Investigate extend operation regression at 10K elements
   - Optimize large type handling for better grow-shrink performance

2. Memory Management:
   - Current clear-and-reuse performance suggests maintaining this pattern
   - Consider tuning growth factors for large vectors
   - Investigate remove-end regression

3. Future Improvements:
   - Implement small vector optimization (SVO) for better small vector performance
   - Consider platform-specific optimizations for large vector operations
   - Explore SIMD opportunities for bulk operations

4. Usage Guidelines:
   - Prefer pre-allocation for known sizes
   - Use clear-and-reuse pattern over grow-shrink when possible
   - Consider vector size and element type when choosing growth strategy 