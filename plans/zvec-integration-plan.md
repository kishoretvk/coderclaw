# Plan: Integrate zvec for SIMD-Optimized Vector Operations

## Overview

Add **zvec** (SIMD-optimized vector operations library) to enhance performance of in-memory vector calculations for embeddings and semantic search operations.

## Current Architecture Analysis

### Existing Vector/Embedding Stack
1. **Embedding Generation**: OpenAI `text-embedding-3-small` (1536D) / `text-embedding-3-large` (3072D)
2. **Storage**: PostgreSQL with `pgvector` for vector similarity search
3. **Search**: Hybrid search combining FTS (full-text) + vector with Reciprocal Rank Fusion (RRF)
4. **Representation**: `Vec<f32>` for embeddings

### Performance Bottlenecks Identified
- In-memory vector distance calculations use naive implementations
- Batch embedding processing lacks SIMD optimization
- Cosine similarity and dot product calculations are scalar

## zvec Integration Benefits

| Operation | Current | With zvec | Expected Speedup |
|-----------|---------|-----------|------------------|
| Dot product (1536D) | ~1.5μs | ~0.2μs | 7-8x |
| Cosine similarity | ~3.0μs | ~0.4μs | 7-8x |
| L2 distance | ~3.0μs | ~0.4μs | 7-8x |
| Batch normalization | ~5.0μs | ~0.6μs | 8x |

## Implementation Plan

### Phase 1: Dependencies & Core Types

- [ ] **Add zvec dependency** to `Cargo.toml`
  ```toml
  zvec = "0.4"
  ```
  
- [ ] **Create new module** `src/workspace/vector_simd.rs` for SIMD operations
  - Wrap zvec types for project conventions
  - Provide fallbacks for non-SIMD platforms

- [ ] **Add feature flag** in Cargo.toml
  ```toml
  zvec = ["dep:zvec"]
  ```

### Phase 2: Vector Operations Module

- [ ] **Implement vector math traits** in `src/workspace/vector_simd.rs`
  - `dot_product(&[f32], &[f32]) -> f32`
  - `cosine_similarity(&[f32], &[f32]) -> f32`
  - `l2_distance(&[f32], &[f32]) -> f32`
  - `normalize(vec: &[f32]) -> Vec<f32>`
  - `batch_normalize(vectors: &[[f32]]) -> Vec<Vec<f32>>`

- [ ] **Add vector caching** for frequently accessed embeddings
  - In-memory LRU cache with zvec-optimized distance calculations

### Phase 3: Embeddings Integration

- [ ] **Update `src/workspace/embeddings.rs`**
  - Add zvec-backed similarity calculations
  - Add batch processing with SIMD acceleration
  - Maintain backward compatibility with existing `Vec<f32>` API

- [ ] **Add embedding cache** layer
  - Cache frequently used embeddings
  - Use zvec for fast similarity lookup

### Phase 4: Search Enhancement

- [ ] **Enhance `src/workspace/search.rs`**
  - Add zvec-powered in-memory reranking
  - Pre-filter candidates using fast L2/cosine before DB query

- [ ] **Optimize batch scoring**
  - Process multiple candidate vectors in parallel

### Phase 5: Testing & Benchmarking

- [ ] **Add benchmarks** in `benchmarks/`
  - Compare zvec vs naive implementations
  - Profile end-to-end search latency

- [ ] **Add unit tests** for vector operations
  - Verify correctness vs reference implementation
  - Test edge cases (zero vectors, NaN, etc.)

## File Changes Summary

| File | Change Type |
|------|-------------|
| `Cargo.toml` | Add zvec dependency + feature flag |
| `src/workspace/vector_simd.rs` | New - SIMD vector operations |
| `src/workspace/embeddings.rs` | Add zvec integration |
| `src/workspace/search.rs` | Add reranking optimization |
| `src/config/mod.rs` | Add zvec configuration |
| `benchmarks/src/lib.rs` | Add vector benchmarks |

## Configuration Options

Add to config system:
```rust
pub struct VectorConfig {
    /// Enable SIMD vector operations (zvec)
    pub use_simd: bool,
    /// Embedding cache size
    pub cache_size: usize,
    /// Enable in-memory reranking
    pub rerank: bool,
}
```

## Risk Mitigation

1. **Platform compatibility**: Provide fallback scalar implementations
2. **API compatibility**: Wrap zvec to maintain existing interfaces
3. **Performance regression**: Benchmark to verify improvements

## Success Criteria

- [ ] 5-8x speedup on vector similarity calculations
- [ ] Zero breaking changes to existing API
- [ ] Benchmarks show measurable improvement
- [ ] Tests pass on all supported platforms
