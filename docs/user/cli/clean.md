<!-- @generated from em's command line; do not edit -->
# em clean

`em clean` — clean distfiles and/or binary packages

**Effect:** destructive (may delete or irreversibly overwrite).

## Usage

```
em clean [OPTIONS] <COMMAND>
```

## Commands

* [`dist`](clean/dist.md) — Remove distfiles no ebuild references
* [`pkg`](clean/pkg.md) — Remove binary packages no ebuild references
* [`all`](clean/all.md) — Everything above, plus the build logs finished merges leave behind

## Options

* `-p, --pretend` — Show what would be done without actually performing any actions
* `-h, --help` — Print help (see a summary with '-h')

## Roots

Which tree this invocation reads and writes.

* `--prefix <DIR>` — Unprivileged offset: ROOT/VDB/distfiles/build trees under DIR; config
  still from the host (use --root for a config offset).
* `--local [<DIR>]` — Unprivileged, standalone Gentoo-Prefix: own VDB/BROOT/config, not
  overlaid on the host (see --prefix for the overlay). Defaults to
  ~/.gentoo (EPREFIX=~/.gentoo) when no DIR is given (`--local=`).

  A bare `--local` takes the next word as DIR.
* `--config-root <PATH>` — Read config (profile, make.conf) from this root instead of `--root`
* `-T, --target <TUPLE>` — Cross-build/setup for a crossdev target tuple

  The single source for "which tuple" everywhere: `em crossdev --target T --init-target`
  sets T up; `em stages --target T --stage1` (or any plain atom build) resolves/installs
  into the target sysroot `<EROOT>/usr/<TUPLE>` — sugar for
  `--config-root <sysroot> --root <sysroot>`.

  Cross context (CHOST/CBUILD, `--root-deps=rdeps`) is read from the
  sysroot make.conf. One flag for both roles — `crossdev` no longer
  has its own `-t`/`--target`.
* `--root <PATH>` — Installation root (the offset an applet installs into / queries)

## Global options

* `--color <WHEN>` — \[default: auto\] \[possible values: auto, always, never\]
