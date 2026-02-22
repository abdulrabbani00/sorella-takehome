# Takeaways

Notes and learnings from the database optimization challenge.

# Implementation

I looked through the codebase to understand what was going on and what was desired. My initial reaction was to mimic what most modern DB's do. I knew I'd have to trim a lot of it down however considering I can't leverage multi-threading and I was constrained by the problem set. A few things that came to mind however:

1. Indexing - For the reads and reloads.
2. Tombstones - Easier to implement than removing records.
3. mmap - This was a no brainer and I knew would have the most upsided returns. 
4. Removing as many syscalls as I could from the hot path and batching writes.

I did this in three general phases.

## Phase 1 - Implement a DB with Custom File Format

The first iteration was a simple DB implementation that would write a custom file to disk which included a header, with offsets highlighting where the next write is and where the index lives. 

I just wanted to get a basic file format that worked and showed improvement.

## Phase 2 - Compact Index + Buffered I/O

I knew seperating out the index into its own file would lead to some nice improvements. Furthermore, I knew buffering my writes and making less syscalls would also lead to performances.

## Phase 3 - mmap

This I knew would be the biggest change, loading the data file and mapping it in-memory would be huge and it was. There are some implementation things to consider, particularly that mmap uses unsafe rust as is its nature. Furthremore, I knew to use random access since data can exist anywhere in the file.

I didn't just right to this however, I wanted to set the foundation to properly implement mmap.

# Use of AI

I used AI in various ways:

- During the planning phase, I knew I wanted indexes, tombstone, and mmap. But I used AI to help me design this approach. Instead of one shotting it I broke it down into three incremental phases that I could benchmark individually.
- Write the code and tracking the results (see below). I reviewed the code but overall was happy with its iterative implementation. I guided it where I didn't like what it was doing. But this is where being very precise with your asks pays off big time. 
- Asking for some potential optimizations. It gave a ton that all led to regressions, the only one that stuck was the `key_str` optimization.

# Results

## Baseline: Default Implementation

- **How measured:** `cargo run --release -- --default-only`
- **Note:** Default is excluded from criterion benchmarks due to excessive runtime. Runner uses fixed ITEMS=1000, REMOVES=500.
- **Date recorded:** 2026-02-22 09:33:32

| Items | Removes | Time |
|-------|---------|------|
| 100 | 50 | N/A |
| 500 | 250 | N/A |
| 1000 | 500 | 19.64 s |
| 10_000 | 5_000 | N/A |


## Step 1: Minimal Viable

- **cargo bench:** Completed (100, 500, 1000). 10_000 run in progress at time of recording.
- **vs default (1000 items):** 19.64 s → 1.05 s ≈ **18.7x faster**

| Items | Removes | Time |
|-------|---------|------|
| 100 | 50 | 111 ms |
| 500 | 250 | 526 ms |
| 1000 | 500 | 1.05 s |
| 10_000 | 5_000 | ~11 s (est.) |

---

## Step 2: Buffered I/O + Compact Index

- **Changes:** BufWriter (64KB), two-file layout (data + path.index), compact binary index format
- **vs default (1000 items):** 19.64 s → 1.02 s ≈ **19.3x faster**
- **vs Step 1:** 100: 111ms → 106ms (~5%), 500: 526ms → 501ms (~5%), 1000: ~same
- **cargo bench:** Completed (100, 500, 1000). 10_000 est. ~10.4s

| Items | Removes | Time |
|-------|---------|------|
| 100 | 50 | 106 ms |
| 500 | 250 | 501 ms |
| 1000 | 500 | 1.02 s |
| 10_000 | 5_000 | ~10.4 s (est.) |

---

## Step 3: mmap for Reads

- **Changes:** memmap2 for read path; mmap data file and slice directly for deserialization (zero read syscalls in hot path)
- **vs default (1000 items):** 19.64 s → 0.31 s ≈ **64x faster**
- **vs Step 2:** 100: 106ms → 37ms (~65%), 500: 501ms → 156ms (~69%), 1000: 1.02s → 0.31s (~70%), 10k: 10.4s → 3.1s (~70%)
- **cargo bench:** Completed all sizes

| Items | Removes | Time |
|-------|---------|------|
| 100 | 50 | 37 ms |
| 500 | 250 | 156 ms |
| 1000 | 500 | 307 ms |
| 10_000 | 5_000 | 3.11 s |

## Final 

- **Changes kept:** key_str manual concat (1.5.5), madvise on mmap (1.5.6)
- **Date:** 2026-02-21
- **vs Step 3:** 3–5% faster across all sizes
- **vs default (1000 items):** 19.64 s → 294 ms ≈ **67× faster**

| Items | Removes | Time | vs Step 3 | vs Default |
|-------|---------|------|-----------|------------|
| 100 | 50 | 35.9 ms | ~3% faster | — |
| 500 | 250 | 147.8 ms | ~5% faster | — |
| 1000 | 500 | 293.7 ms | ~4% faster | **67× faster** |
| 10_000 | 5_000 | 3.03 s | ~2.6% faster | — |


# Final Thoughts

There are likely some other changes we can make to get some minor improvements. But I knew mmap improvements would outweight just about everything else. So I wanted to get the foundation setup prior to implementing it and then focusing on optimizing for it.

I appreciate the thoughtful exercise to show what I can do in a real world setting.
