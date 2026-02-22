# Takeaways

Notes and learnings from the database optimization challenge.

---

## Potential Changes

Ideas considered but not implemented in the current solution. May be worth revisiting if constraints change or for future iterations.

### Cow for Zero-Copy Deserialization

**Idea:** Use `Cow<'a, str>` and `Cow<'a, [T]>` in `StoredType` and `RandomNestedStructure` instead of `String` and `Vec`, enabling zero-copy deserialization from the mmap slice. The TypeGenerator would produce `Cow::Owned(...)`; the database would return `Cow::Borrowed(...)` when deserializing with `serde_json::from_slice` and serde’s borrow support.

**Why not pursued:** The README explicitly allows changing `String` to `&str` and `Vec` to `&[]`, but does not mention `Cow`. To stay strictly within the written rules, we avoided Cow and kept owned types.

**If allowed:** Would require updating `types.rs`, `CandidateTypeGen`, and the database read path. Likely improves read-path performance by avoiding per-read allocation of strings and vectors.
