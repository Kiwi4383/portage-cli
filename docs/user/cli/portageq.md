<!-- @generated from em's command line; do not edit -->
# em portageq

`em portageq` — a subset of Portage's own query interface (byte-compatible
where implemented; only the installed-package query group exists)

**Effect:** read-only.

## Usage

```
em portageq [OPTIONS] <COMMAND>
```

## Commands

* [`has_version`](portageq/has_version.md) — Exit 0 if ATOM is installed under EROOT, 1 otherwise
* [`best_version`](portageq/best_version.md) — Print the highest installed version matching ATOM under EROOT
* [`match`](portageq/match.md) — List every installed CPV matching ATOM under EROOT, one per line
* [`mass_best_version`](portageq/mass_best_version.md) — Print `atom:cpv` for each of several atoms' best installed version
* [`envvar`](portageq/envvar.md) — Print profile variable values by name

## Options

* `-h, --help` — Print help (see a summary with '-h')

## Global options

* `--color <WHEN>` — \[default: auto\] \[possible values: auto, always, never\]

## Exit status

* `0` — query succeeded
* `1` — no match / unset variable
* `2` — invalid atom
* `64` — \<EROOT\> is not a directory

## Examples

```
$ em portageq has_version / sys-apps/portage
```
