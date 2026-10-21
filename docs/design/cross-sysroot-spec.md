# Cross-sysroot spec: what it says and how it becomes Portage config

A cross target's description is data: `SysrootSpec` (`portage-cli/src/crossdev/spec.rs`), serialised as
TOML. A tuple has a built-in spec (`em crossdev --target T --print-spec`); `--target` also takes a spec file.
This page states how a spec turns into files Portage and `em` read, and what is still missing.

> **Slop warning.** Verify against the code before relying on this page; update it when it drifts.

## The two conversions

A spec cannot become one kind of file. Portage reads some settings from a profile and others only from the
user configuration directory, so a spec converts in two places:

| Spec says | Lands in | Why there |
|---|---|---|
| `profile` | the derived profile's `parent`, as `gentoo:<profile>` | a profile is how Gentoo describes a system |
| a version or slot on a toolchain package (a pin) | the derived profile's `package.mask` + `package.unmask` | both are profile files |
| which toolchain builds the system (`system.built-by`) | the derived profile's `make.defaults` (planned) | any variable may be set there |
| packages another toolchain builds (`system.exceptions`) | `<sysroot>/etc/portage/package.env` + `env/` (planned) | Portage reads `package.env` only from user config |
| `cflags` | `<sysroot>/etc/portage/make.conf` today; the profile's `make.defaults` is the natural place | |

Built-in tuples get neither: their `make.profile` links the `::gentoo` profile directly, as before specs.

## Conversion 1: the derived profile (implemented)

`--init-target` with a spec file writes an overlay inside the sysroot and links `make.profile` to its profile:

```
<sysroot>/var/db/repos/cross-sysroot/
  metadata/layout.conf          masters = gentoo, profile-formats = portage-2
  profiles/repo_name            cross-sysroot
  profiles/sysroot/eapi         8
  profiles/sysroot/parent       gentoo:<spec.profile>
  profiles/sysroot/package.mask      one bare package per pin
  profiles/sysroot/package.unmask    the pins as written
  profiles/sysroot/cross-sysroot.toml   the spec itself
<sysroot>/etc/portage/repos.conf/cross-sysroot.conf
<sysroot>/etc/portage/make.profile -> .../profiles/sysroot
```

- **`gentoo:` parents** are Portage's `portage-2` profile format; PMS only defines relative `parent` paths.
  `em` reads them through `ProfileStack::build_in`, resolving the name through the config root's `repos.conf`.
- **The `eapi` file is required.** Without it Portage reads the profile as EAPI 0, which has no slot
  dependencies, and drops a pin such as `sys-devel/gcc:15` from `package.unmask`.
- **`package.unmask` in a profile** is read by Portage only from profiles allowing `portage-1` directories
  (outside a repository, the user profile, or a `portage-1`/`portage-1-compat`/`portage-2` repository); `em`
  applies the same gate.
- **The saved spec** is what a later command given just the tuple uses: it follows `make.profile` and loads
  `cross-sysroot.toml`.
- **The cross packages** (`cross-<tuple>/*`, built on the host) cannot see the sysroot profile. Their pins go to
  the host's `package.mask/<category>` and `package.unmask/<category>`, and the keyword line stops below the
  release branch of the newest keyworded version a pin allows, so a live ebuild never wins.

## Conversion 2: per-package toolchains (planned)

What the clang-as-system-compiler experiment configured by hand, written by `--init-target` instead:

- **The default toolchain** sets the compilers for the whole target, tuple-prefixed, e.g.
  `CC="<tuple>-clang" CXX="<tuple>-clang++" CPP="<tuple>-clang-cpp"`. A GCC default needs nothing: the
  `<tuple>-gcc` default already applies. This belongs in the derived profile's `make.defaults`, which
  `make.conf` can still override.
- **Each other toolchain** gets an env file, `<sysroot>/etc/portage/env/<toolchain>-toolchain.conf`
  (`CC=<tuple>-gcc`, `CXX=<tuple>-g++`, `LD=<tuple>-ld`, …).
- **Each exception** gets a `package.env` line under `<sysroot>/etc/portage/package.env/`, e.g.
  `sys-libs/glibc gcc-toolchain.conf`. Portage has no profile form of `package.env` (it reads it from
  `abs_user_config` only), so this stays in the sysroot's `/etc/portage`, regenerated like `make.conf`.

It is only useful with a spec defining both toolchains, which loading refuses until the bootstrap can build
that combination: gcc, then glibc, then compiler-rt, libunwind, libcxxabi and libcxx, then the clang
configuration switched to the LLVM runtimes. The two land together.

## Where the spec has to grow

The experiment needed settings the spec cannot state yet. Each is a candidate field, converted by the same rule
(profile file where Portage allows one, user config where it does not):

- **More than one parent profile.** No LLVM profile exists for riscv; the sysroot needed the arch profile plus
  `features/llvm`. `profile` becomes a list, written as several `parent` lines.
- **Per-toolchain variables.** Which tools each toolchain sets (`LD`, `AR`, `NM`, `OBJCOPY` for LLVM's tools),
  instead of a fixed set in code.
- **The clang driver configuration.** `/etc/clang/cross/<tuple>.cfg` (`--sysroot`, `--target`, `--rtlib`,
  `--unwindlib`, `--stdlib`, `-fuse-ld`) whenever a clang toolchain is defined, not only for `-L`; derived from
  the clang roles.
- **Per-package USE the toolchain needs**, such as `static-libs` on the LLVM runtimes: the derived profile's
  `package.use`. Stage1's `USE="-* build"` still drops profile USE, so stage1 needs its own answer.

The bootstrap order stays derived in code (`crossdev/stages.rs`) from who builds what, not written in the spec.
