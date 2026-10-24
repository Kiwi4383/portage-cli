<!-- @generated from em's command line; do not edit -->
# em portageq envvar

Print configuration variables as a build would see them

## Usage

```
em portageq envvar [OPTIONS] <NAME>...
```

## Arguments

* `<NAME>...` — A trailing `*` matches every variable with that prefix

## Options

* `-v` — Print `NAME=value` lines, quoted for the shell
* `-h, --help` — Print help (see a summary with '-h')

## Global options

* `--color <WHEN>` — \[default: auto\]
  * `auto`: Colour when stdout is a terminal.
  * `always`: Always colour.
  * `never`: Never colour.
