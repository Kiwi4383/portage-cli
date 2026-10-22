# FEATURES parity

Status: 🔴 not started, filed 2026-10-22 from [[portage-config-layering]] (decision D3).
Related: [[build-clean-env]] (the `keepwork`/`keeptemp`/`noclean` subset, done).

## Decision (Luca, 2026-10-22)

The ebuild environment's `FEATURES` lists what `em` provides, not make.globals' set: an
eclass testing `has network-sandbox ${FEATURES}` must not be told about a sandbox em does not
run. Matching Portage's set is this note's work, one feature at a time.

## Where em stands (grep of `features.contains(…)`, 2026-10-22)

- **Read from FEATURES:** `binpkg-request-signature`, `binpkg-signing`, `candy`, `ccache`,
  `keeptemp`, `keepwork`, `noclean`, `nostrip`, `test`, `test-fail-continue`; `sandbox` only in
  a test.
- **Make.globals defaults em has no FEATURES check for** (31): `assume-digests`,
  `binpkg-docompress`, `binpkg-dostrip`, `binpkg-logs`, `binpkg-multi-instance`,
  `buildpkg-live`, `compress-index`, `config-protect-if-modified`, `distlocks`, `ebuild-locks`,
  `fixlafiles`, `ipc-sandbox`, `merge-sync`, `merge-wait`, `multilib-strict`, `network-sandbox`,
  `news`, `parallel-fetch`, `pkgdir-index-trusted`, `pid-sandbox`, `preserve-libs`,
  `protect-owned`, `qa-unresolved-soname-deps`, `sandbox`, `strict`, `unknown-features-warn`,
  `unmerge-logs`, `unmerge-orphans`, `userfetch`, `userpriv`, `usersandbox`, `usersync`.

## Work

1. Sort the 31 into: behaviour em always has (no switch: likely `preserve-libs`,
   `binpkg-multi-instance`, the collision checks), behaviour em lacks (the sandbox family,
   `userpriv`/`userfetch`/`usersync`, `distlocks`), and features with no meaning for em. Check
   each against the code, not the name.
2. Export to the ebuild environment the ones em provides, always-on ones included.
3. For each always-on one, decide whether `-feature` in make.conf must switch it off, as in
   Portage.
4. `em --info`/`portageq envvar FEATURES` show Portage's stacked value (make.globals included)
   once [[portage-config-layering]] step 2 exists; the build environment shows em's. Say so in
   `--info` if the two differ.
