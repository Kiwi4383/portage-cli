# Changelog

## 0.11.0

### Breaking changes

- Depends on `portage-atom` 0.12.
- `LICENSE`: a bare `( … )` group is a nested `LicenseExpr::All`, written back with its parentheses, instead
  of being spliced into its parent; `|| ( MIT ( GPL-1 GPL-2 ) )` no longer reads as "MIT or GPL-1 or GPL-2".

### Features

- PMS table queries on `Eapi`.
- A malformed lazy field is told apart from an empty one.

### Bug fixes

- `SRC_URI`: parentheses inside a filename are part of it.
- Dependency fields and `SRC_URI` serialise from their parsed content.

### Performance

- Dependency fields and `SRC_URI` parse lazily; eclass checksums are interned and typed; cache entries
  serialise as a stream.

## 0.10.0

### Breaking changes

- `Error::Invalid{SrcUri,License,RequiredUse,Restrict}` now carry
  `ParseDiagnostic` instead of `String`. Use `Error::parse_diagnostic()` (or
  the variant payload) and render as a `miette::Diagnostic` in the application;
  the library no longer pre-renders colorized strings.

### Features

- Structured `ParseDiagnostic` (`miette::Diagnostic`) for `SRC_URI` / `LICENSE` /
  `REQUIRED_USE` / `RESTRICT` parse failures, with a cropped source window
  around the failing span.

### Bug fixes

- Accept `!` in `SRC_URI` tokens (needed by live trees such as pentoo).
- Never emit ANSI color from library diagnostic paths (CLI owns rendering).

### Other

- Depend on `miette` for structured diagnostics.
- Packaging hygiene: exclude workspace-only files from the crates.io tarball.

## 0.9.0

### Performance

- Stop round-tripping already-interned `IUSE` keys through the interner on the
  per-version hot path (`From<&IUse<_>> for Interned<_>`).

### Other

- Inherit `rust-version` and `repository` from the workspace package metadata.

## 0.8.0

### Breaking changes

- Depend on `portage-atom` 0.10 (which changed `DepEntry::evaluate_use`); the
  major bump is propagated to dependents.

### Features

- Evaluate `REQUIRED_USE` expressions.
- Collect `SRC_URI` distfile names for a given USE state.
- Apply `IUSE` defaults for merge-path parity.
- Expose interned `IUSE` keys without re-interning.

### Documentation

- Document all public items and enable `#![warn(missing_docs)]`.

### Other

- Raise MSRV to 1.92.
