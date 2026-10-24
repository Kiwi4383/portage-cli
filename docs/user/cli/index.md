<!-- @generated from em's command line; do not edit -->
# em

Gentoo Portage package manager workalike

## Usage

```
em [OPTIONS] [COMMAND]
```

## Commands

* [`ebuild`](ebuild.md) — Execute ebuild phases
* [`maint`](maint.md) — System maintenance and health checks
* [`portageq`](portageq.md) — Query Portage internal variables and data
* [`sync`](sync.md) — Sync repositories (git, rsync)
* [`depclean`](depclean.md) — Remove orphaned/unused packages
* [`regen`](regen.md) — Regenerate metadata cache
* [`quickpkg`](quickpkg.md) — Create binary packages from installed files
* [`mirrordist`](mirrordist.md) — Build/maintain a distfiles mirror (emirrordist workalike)
* [`query`](query.md) — Query package information
* [`clean`](clean.md) — Clean distfiles and/or binary packages
* [`use`](use.md) — Enable/disable/query USE flags in make.conf
* [`pkg`](pkg.md) — Edit per-package configuration (package.use, .keywords, .mask, .env)
* [`revdep`](revdep.md) — Rebuild packages with broken shared library deps
* [`read`](read.md) — Display Portage elog files
* [`log`](log.md) — Analyze emerge.log
* [`grep`](grep.md) — Search inside ebuilds and eclasses
* [`search`](search.md) — Search package names and descriptions
* [`atom`](atom.md) — Parse/split atom strings
* [`select`](select.md) — Native config selectors (profile, repos) — eselect-like
* [`active`](active.md) — Register a default --prefix/--local for bare em invocations
* [`setup`](setup.md) — Bootstrap a prefix layout (use with --local or --prefix)
* [`crossdev`](crossdev.md) — Set up a cross-compilation target (sysroot + overlay) — crossdev workalike
* [`toolchain`](toolchain.md) — Bootstrap a self-hosting native toolchain (--root / --prefix / --local)
* [`stages`](stages.md) — Assemble stage-build artifacts (stage1 packages.build) into --root
* [`etc`](etc.md) — Reconcile pending config files (etc-update / dispatch-conf)
* [`env`](env.md) — Regenerate /etc/profile.env and ld.so cache
* [`completion`](completion.md) — Print a shell completion script
* [`emerge`](emerge.md) — Resolve and merge/unmerge packages (emerge workalike) (default)

## Options

* `--color <WHEN>` — \[default: auto\]
  * `auto`: Colour when stdout is a terminal.
  * `always`: Always colour.
  * `never`: Never colour.
* `-h, --help` — Print help (see a summary with '-h')
* `-V, --version` — Print version

## Examples

**Update @world**

```
$ em -ua @world
```

**Find a file's owner**

```
$ em query belongs /usr/bin/python
```

**List profiles**

```
$ em select profile list
```
