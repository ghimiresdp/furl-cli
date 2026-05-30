# ADR-0001: Progress reporting via trait and optional feature gates

**Date decided:** 2025-05-27
**Status:** Proposed
**Author:** Sudip Ghimire

---

## Context

`furl-cli` exposes a library target (`furl_core`) alongside its CLI
binary so that other programs — such as game engines downloading
assets in parallel — can embed the download engine without
reimplementing multithreaded HTTP chunking.

However, the library currently has two problems that make embedding
it harder than it should be:

- `indicatif` is used directly inside the engine (`src/engine.rs`),
  meaning all library consumers compile and inherit a terminal
  progress bar dependency regardless of whether they have a terminal
  at all.
- `clap` is in the shared `[dependencies]` block, so consumers also
  pull in an argument parsing library that is only relevant to the
  CLI binary.

A game engine, build tool, or headless service embedding `furl_core`
has no use for either. They may also have their own progress UI that
conflicts with or duplicates `indicatif`'s output.

---

## Decision

### 1. Introduce a `ProgressReporter` trait in the library core

A new public trait is added with no feature gate — always available:

```rust
pub trait ProgressReporter: Send + Sync {
    fn on_start(&self, total_bytes: u64);
    fn on_progress(&self, bytes_downloaded: u64);
    fn on_finish(&self);
}
```

The engine accepts a `Arc<dyn ProgressReporter>` instead of
holding an `indicatif` handle directly. All progress-related
calls inside the download loop go through this trait.

A `NoopReporter` (no-op implementation) is always provided for
consumers who do not want any progress output.

### 2. Gate `indicatif` behind a `progress` feature

An `IndicatifReporter` implementing `ProgressReporter` is provided
in a `progress` module, compiled only when the `progress` feature
is enabled. Consumers who want the built-in terminal progress bar
opt in explicitly.

### 3. Gate `clap` behind a `cli` feature

Argument parsing in `main.rs` is compiled only when the `cli`
feature is enabled.

### 4. Both features are on by default

```toml
[features]
default = ["cli", "progress"]
cli      = ["dep:clap"]
progress = ["dep:indicatif"]
```

Existing CLI users (`cargo install furl-cli`) and anyone using
`furl-cli` as a dependency without `default-features = false`
are completely unaffected.

Library consumers who want a lean dependency tree opt out
explicitly:

```toml
# core only — no indicatif, no clap
furl-cli = { version = "0.x", default-features = false }

# with built-in progress bars
furl-cli = { version = "0.x", default-features = false,
             features = ["progress"] }
```

---

## Consequences

### Positive

- **Dependency opt-out.** Library consumers with no terminal
  (game engines, GUI apps, headless servers) no longer compile
  `indicatif` or `clap` unless they explicitly ask for them.

- **Custom progress UI.** Any consumer can implement
  `ProgressReporter` with their own UI — a GUI loading bar,
  a logging call, a metrics counter — without fighting the
  library's built-in output.

- **No breaking change for existing users.** Default features
  preserve the current behaviour entirely.

- **Cleaner API boundary.** The engine core has zero knowledge
  of terminal output. Progress reporting is the caller's concern;
  the library provides a good default, not a mandate.

### Negative / Trade-offs

- **Slightly more complex API.** `Downloader::new()` now requires
  a reporter argument. Convenience constructors (`::silent()`,
  `::with_progress()`) reduce this friction but add surface area.

- **`cfg` gates add noise.** Feature-gated code paths require
  `#[cfg(feature = "...")]` annotations throughout, which adds
  minor cognitive overhead when reading the source.

- **`indicatif` state threading.** `indicatif`'s `ProgressBar`
  is already `Send + Sync` so the trait bound is satisfied without
  extra wrapping. If a future reporter implementation is not
  trivially thread-safe, callers bear responsibility for that.

---

## Alternatives considered

### A: Keep `indicatif` in the engine, document it as a feature

Leave the current structure, add a note in the README that
consumers accept `indicatif` as a transitive dependency.

**Rejected because:** it permanently closes the door on custom
progress UIs and forces an unnecessary compile-time dependency
on every embedder. As the library gains more consumers, this
becomes increasingly painful to undo.

### B: Remove progress reporting from the library entirely

Strip all `indicatif` usage from the engine. Let CLI `main.rs`
manage the progress bar by polling download state externally.

**Rejected because:** download progress is inherently tied to
the chunked download loop inside the engine. Exposing enough
internal state for an external caller to reconstruct progress
accurately would require a more complex event stream API. The
trait approach gives callers full control with less surface area.

### C: Use a callback closure instead of a trait

Accept `Box<dyn Fn(u64) + Send + Sync>` per-event instead of a
structured trait.

**Rejected because:** separate named methods (`on_start`,
`on_progress`, `on_finish`) carry semantic meaning — a consumer
implementing the trait knows exactly what each hook represents.
A closure-per-event API fragments that contract and makes the
no-op case more verbose.

---

## References

- Cargo features reference:
  <https://doc.rust-lang.org/cargo/reference/features.html>
- `indicatif` crate: <https://docs.rs/indicatif>
- `clap` crate: <https://docs.rs/clap>
