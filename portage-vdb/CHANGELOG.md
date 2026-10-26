# Changelog

## 0.3.0

### Breaking changes

- Depends on `portage-atom` 0.12 and `portage-metadata` 0.11.
- `MergeSpec` gains `iuse_effective`; `ContentsKind` is now `Copy`.
- `InstalledPackage` returns typed values: `slot()` a `Slot` (the text is `slot_raw()`), `slot_main()` and
  `subslot()` a `SlotName`, `iuse()` `IUse`, and `use_flags()`, `keywords()` and `repository()` interned
  names instead of `String`s.

### Features

- `contents_into` reads `CONTENTS` into a caller's buffer, `contents_raw` returns its text, and `scan` walks
  the database in parallel.

### Bug fixes

- Registering a package stages through `-MERGING-` instead of writing in place.
- A package's field cache is cleared in place, so stale handles recover.

### Performance

- `CONTENTS` stays out of the field cache; field caching is scoped; the ownership index is reused across
  merge batches.

## 0.2.1

### Other

- Ship a `LICENSE` file; exclude local `Cargo.lock` from the crates.io tarball.

## 0.2.0

### Features

- Richer VDB metadata field set (closer to Portage's on-disk keys).
- ELF link metadata scan (`NEEDED` / `REQUIRES` / `PROVIDES`) and ebuild copy
  into the VDB entry.
- Record `RUSTFLAGS` through the producer path used by binpkg / Packages.
- Field cache for flat-file reads (invalidated per entry); poison-tolerant under
  concurrent workers.

### Bug fixes

- Parse `CONTENTS` paths that contain spaces.
- Loud, non-silent failures when worker env setup fails.

### Other

- Inherit `rust-version` and `repository` from the workspace package metadata.

## 0.1.0

- Initial VDB reader/writer: open `/var/db/pkg`, package iteration, ownership
  lookup, collision detection, register/unregister, `camino` paths, lazy
  iterators.
