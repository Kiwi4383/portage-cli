<!-- @generated from em's command line; do not edit -->
# em atom

`em atom` — parse/split atom strings

**Effect:** read-only.

## Usage

```
em atom [OPTIONS] <ATOMS>...
```

## Arguments

* `<ATOMS>...` — Atom strings to parse and print back in normalized form

## Options

* `-h, --help` — Print help (see a summary with '-h')

## Global options

* `--color <WHEN>` — \[default: auto\]
  * `auto`: Colour when stdout is a terminal.
  * `always`: Always colour.
  * `never`: Never colour.

## Examples

```
$ em atom >=dev-lang/python-3.10
```
