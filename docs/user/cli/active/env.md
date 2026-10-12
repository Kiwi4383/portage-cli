<!-- @generated from em's command line; do not edit -->
# em active env

Print shell exports for the active context

## Usage

```
em active env [OPTIONS]
```

## Options

* `-h, --help` — Print help (see a summary with '-h')

## Global options

* `--color <WHEN>` — \[default: auto\] \[possible values: auto, always, never\]
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
