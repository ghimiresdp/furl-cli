# ADR-0002: Cargo workspace for example crates

**Date decided:** 2026-05-27
**Status:** Proposed
**Author:** Sudip Ghimire

---

## Context

`furl-cli` is currently structured as a single Cargo package that
exposes both a library target (`furl_core`) and the `furl` CLI
binary.  ADR-0001 introduced optional feature gates so that library
consumers can embed `furl_core` with a lean dependency tree.

To demonstrate real-world embedding patterns — a headless downloader,
a game-asset fetcher, or a custom progress UI — the project needs
runnable example programs.  Cargo supports two ways to provide these:

- **`[[example]]` targets** inside the main `Cargo.toml`.  These are
  compiled as part of the `furl-cli` package and share its
  dependency set.  They cannot declare their own dependencies.
- **Separate workspace member crates** under an `examples/`
  directory.  Each is an independent Cargo package and can have its
  own `Cargo.toml`, dependencies, and entry points.

Using `[[example]]` targets is fine for trivial snippets, but limits
what can be demonstrated.  An example that uses a GUI progress bar
or a game-engine asset system would need to pull in large external
crates — inappropriate to add to the main package manifest.

---

## Decision

### 1. Add a `[workspace]` section to the root `Cargo.toml`

A `[workspace]` table is added to the existing root `Cargo.toml`.
`furl-cli` remains the package at the workspace root and is
implicitly included as a workspace member; no source files or
directories are moved.

```toml
[workspace]
members  = ["examples/*"]
resolver = "2"
```

### 2. Add example crates as workspace members

Each non-trivial example lives in its own subdirectory under
`examples/`, with its own `Cargo.toml` that declares `furl-cli` as
a dependency (typically with `default-features = false` to
demonstrate the lean embedding use-case).

```text
examples
└── embedded-minimal
    ├── Cargo.toml
    └── src
        └── main.rs

```

Simple, self-contained snippets that require no extra dependencies
may continue to use `[[example]]` targets inside the main package.

### 3. `furl-cli` remains the only published crate

The root `furl-cli` package is the only workspace member published
to crates.io.  Every example crate sets `publish = false` in its
own `Cargo.toml` to prevent accidental publishing.

```toml
# examples/headless-download/Cargo.toml
[package]
name    = "headless-download"
version = "0.1.0"
edition = "2024"
publish = false
```

---

## Consequences

### Positive

- **Independent dependencies.** Example crates can pull in their own
  crates (GUI toolkits, logging frameworks, async runtimes) without
  polluting the `furl-cli` dependency tree or inflating compile
  times for end-users.

- **Runnable, realistic examples.** Each example is a fully
  buildable binary that consumers can clone and adapt, rather than
  a code snippet that must be manually wired into a project.

- **Foundation for future crates.** The workspace structure makes it
  straightforward to add further members later (e.g. a standalone
  `furl-core` crate, integration test harnesses) without
  restructuring the repository again.

- **Shared dependency resolution.** Cargo deduplicates dependencies
  across all workspace members, so a shared transitive dependency
  such as `tokio` is compiled once and reused.

- **No disruption to existing users.** The `furl-cli` package name,
  version, and published API are unchanged.  `cargo install furl-cli`
  continues to work without modification.

### Negative / Trade-offs

- **`cargo publish` must target the root package explicitly.**
  Running `cargo publish` from the workspace root without
  `--package furl-cli` will error.  CI publish steps must use
  `cargo publish -p furl-cli`.

- **`Cargo.lock` is shared.** All workspace members share a single
  lock file.  An example that pins an unusual version of a shared
  dependency can cause unexpected resolution for the main crate
  during local development.  This is a local-only concern; published
  crates do not ship the lock file.

- **Slightly more directory structure.** Contributors need to be
  aware that `examples/` contains full Cargo packages rather than
  plain source files, and should run `cargo build --workspace` to
  build everything at once.

---

## Alternatives considered

### A: Keep `[[example]]` targets in the main `Cargo.toml`

Add example source files under `examples/` (Cargo's conventional
path) and declare them as `[[example]]` entries in the main package.

**Rejected because:** `[[example]]` targets share the package
dependency set.  Any crate needed only for an example (a GUI
toolkit, an alternative async runtime, etc.) would become a
dependency of `furl-cli` itself, increasing compile times and the
published crate's dependency surface for all users.

### B: Host examples in a separate repository

Maintain a companion `furl-cli-examples` repository with its own
`Cargo.toml` that depends on the released `furl-cli` crate.

**Rejected because:** it creates synchronisation overhead — examples
must be updated separately whenever the library API changes, and
contributors cannot validate an API change alongside its example in
a single pull request.  A workspace keeps both in lockstep.

---

## References

- Cargo workspaces reference:
  <https://doc.rust-lang.org/cargo/reference/workspaces.html>
- Cargo `[[example]]` targets:
  <https://doc.rust-lang.org/cargo/reference/cargo-targets.html#examples>
- ADR-0001 (optional feature gates):
  [0001-optional-progress-reporting.md](0001-optional-progress-reporting.md)
