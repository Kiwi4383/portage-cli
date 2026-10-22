# One resolved Portage configuration, instead of one per consumer

Status: 🔴 planned 2026-10-22, decisions taken the same day, nothing implemented. Started from `em portageq envvar` not
matching `portageq envvar`; the diff led to a live resolver bug (below). Related:
[[pms-env-unset]], [[use-order-repo-layer]], [[use-stable-in-defaults]].

## Evidence

`portageq envvar -v '*'` against `em portageq envvar -v '*'` on thalia (2026-10-22): about 140
variables against about 120, roughly 70 differing in name or value. `USE` is identical.

The one that changes behaviour: make.conf has `ACCEPT_LICENSE="* -@EULA"`, the base profile
`-* @FREE`. em stacks them like FEATURES, which drops a `-token` with nothing to remove, and
ends with `@FREE *`. `emerge -p app-benchmarks/geekbench` refuses it ("masked by: geekbench
license(s)"); `em -p` plans `[ebuild N] app-benchmarks/geekbench-7.1.0`.

## What Portage builds (portage 3.0.82, `package/ebuild/config.py`, read for understanding)

Layers, lowest first: `env.d` (`/etc/profile.env`), `repo` (repos' `profiles/make.defaults`),
`features`, `globals` (`<EPREFIX>/usr/share/portage/config/make.globals`), `defaults` (profile
stack `make.defaults`), `conf` (make.conf), `pkg`, `env` (the process environment). A plain
lookup takes the highest layer that has the variable.

- **Incrementals** (`const.INCREMENTALS`: ACCEPT_KEYWORDS, CONFIG_PROTECT(_MASK), ENV_UNSET,
  FEATURES, IUSE_IMPLICIT, PROFILE_ONLY_VARIABLES, USE, USE_EXPAND, USE_EXPAND_HIDDEN,
  USE_EXPAND_IMPLICIT, USE_EXPAND_UNPREFIXED): every layer's tokens stacked as a set (`-*`
  clears, `-x` removes), printed sorted.
- **ACCEPT_LICENSE / ACCEPT_PROPERTIES / ACCEPT_RESTRICT**: not set-stacked. Tokens from every
  layer concatenated, then cut back to the last `*` or `-*`; negations such as `-@EULA` stay.
  An empty ACCEPT_LICENSE falls back to `* -@EULA`; ACCEPT_PROPERTIES defaults to `*`.
- **USE_EXPAND variables** are rebuilt from the final USE (`CPU_FLAGS_ARM` from `use.force`'s
  `cpu_flags_arm_*`); a group with no flags is left out unless something set it.
- **Derived**: ROOT, EROOT, SYSROOT, ESYSROOT, EPREFIX, BROOT, PORTAGE_CONFIGROOT from the
  roots; PORTDIR/PORTDIR_OVERLAY/PORTAGE_REPOSITORIES from repos.conf; PORTAGE_ARCHLIST from
  `arch.list`; CBUILD, USERLAND, PORTAGE_GID/USERNAME and the like.
- **Filtering**: PROFILE_ONLY_VARIABLES (ARCH, ELIBC, KERNEL, …) are dropped from make.conf and
  the environment; `env_blacklist` (ebuild metadata names) is dropped from every layer;
  CONFIG_PROTECT is global-only.

## What em does today: each consumer assembles its own

| Consumer | Layers it reads |
|---|---|
| `apply_profile_env` (build shell, `--info`, `envvar`, pkg/use output) | profile stack + make.conf, sourced into a shell; `INCREMENTAL_VARS` merged with `merge_use_var` (drops lone `-x`) |
| `use_env::build_use_env` (resolver) | its own profile/make.conf read; ACCEPT_LICENSE from the shell, replaced wholesale by the environment's; USE has a real env layer |
| `ebuild/protect.rs` | make.globals + `/etc/env.d` + profile, assembled by hand |
| `ebuild::elog_setting` | environment, then shell, then make.globals |
| `binpkg.rs` (PKGDIR) | its own chain with make.globals for host builds |
| `info::resolve_info_var` | LANG from the environment, four names through `elog_setting` |

So: no make.globals except in three hand-written fallbacks, no `env.d` except in protect.rs,
the environment only for USE and ACCEPT_LICENSE (the latter as a replacement, not a layer),
ACCEPT_* stacked with the wrong rule, no USE_EXPAND rebuild, no derived variables, and the
`INCREMENTAL_VARS` list differs from Portage's (has ACCEPT_LICENSE; lacks ENV_UNSET,
PROFILE_ONLY_VARIABLES).

## Plan

Each step lands on its own, with a parity check against the real tool on thalia.

1. **ACCEPT_LICENSE, now.** Stack ACCEPT_LICENSE/PROPERTIES/RESTRICT by Portage's rule (concat,
   cut at the last `*`/`-*`, keep negations), with the environment as the top layer rather
   than a replacement, and `* -@EULA` / `*` when empty. Take ACCEPT_LICENSE out of
   `INCREMENTAL_VARS`. Test: a unit test on the stacking, and `em -p app-benchmarks/geekbench`
   refused as emerge refuses it.
2. **A `Settings` value** (see "Settings: home and lifecycle"). Typed lookup, the incremental
   and ACCEPT_* rules above, the USE_EXPAND rebuild from the USE the profile code already
   resolves correctly. Each value keeps which layer it came from (for `--info -v` and
   debugging). Test: an ignored live test diffing `portageq envvar -v` for every name, in a
   clean environment and with `CFLAGS=…`/`FEATURES=…` set, plus `PORTAGE_CONFIGROOT` scratch
   roots for the `-*`/`-x`/profile-only cases. The derived ROOT/EROOT/… are not compared
   here; step 3 adds them.
3. **`portageq envvar` and `--info` read `Settings`.** The wildcard enumerates settings, not
   shell variables (no BASHPID/IFS/`__PORTAGE_ECLASS_DIRS`, PORTDIR included). `portage-cli`
   adds ROOT, EROOT, SYSROOT, ESYSROOT, EPREFIX, BROOT and PORTAGE_CONFIGROOT from `Roots`.
4. **The resolver reads `Settings`** for ACCEPT_KEYWORDS/LICENSE/PROPERTIES/RESTRICT,
   replacing `use_env`'s own make.conf and environment reads. `-p` parity sweep
   (`bench-em-vs-emerge.sh`, resolve-mode matrix) before and after.
5. **Retire the hand-assembled readers**: protect.rs, `elog_setting`, binpkg PKGDIR,
   `resolve_info_var` all read `Settings`. Merge-time CONFIG_PROTECT parity checked against a
   real merge.
6. **The build environment:** the shell is seeded from `Settings` instead of re-sourcing, with
   FEATURES limited to what em provides (D3); still no process environment (D2).

## Decisions (Luca, 2026-10-22)

- **D1 — no make.globals:** postponed. The globals layer is the file when it exists and empty
  otherwise; which defaults em gives a host without one is decided later.
- **D2 — the environment in builds:** later. The build shell stays clean; Portage's override
  (`CFLAGS=-O0 emerge foo` builds with `-O0`) is a separate item below. The environment is
  still a `Settings` layer, for config decisions and display.
- **D3 — FEATURES in the ebuild environment:** what em provides, not make.globals' set.
  Parity is [[features-parity]].
- **D4 — crate home:** `portage-repo`, beside `ProfileStack` and the make.conf parser. It does
  not need `Roots`: ROOT, EROOT, SYSROOT and the rest are the caller's topology, not a config
  layer, so `portage-cli` adds them where it shows or exports them. (The first draft said
  `Settings` needs both crates; it does not.)

## Settings: home and lifecycle

- **Inputs, all resolved by the caller:** the config root, its profile stack, the make.globals
  and env.d paths, the `--prefix` overlay, one snapshot of the process environment, and a
  `PendingConfig` standing in for files `--init-target` has not written. Nothing below reads
  `std::env` or picks paths on its own.
- **Built once per config root per invocation**, then immutable. A plain run has one; crossdev
  has the host's, the sysroot's and the outer root's. A per-invocation cache keyed by config
  root hands out shared references, like `VdbSnapshotCache`.
- **Evaluation once:** make.defaults and make.conf are evaluated with the shell em already
  uses (`ProfileStack::profile_env`), and `Settings` keeps the result as data. Consumers never
  re-source the files.
- **Consumers derive, never mutate:** the resolver builds its `UseEnv` from it; a build session
  seeds its shell from it; per-package layers (`package.env`, `package.use`, `pkg`) are views
  over it. Portage's `setcpv` mutates one shared config per package; em keeps the base fixed.

## Later

- The environment in build phases, filtered as Portage's `environ_filter` does (D2).
- make.globals defaults for hosts without Portage (D1).

## Not in scope here

USE_ORDER's `repo` layer ([[use-order-repo-layer]]), `use.stable`
([[use-stable-in-defaults]]), applying ENV_UNSET to phases ([[pms-env-unset]]): they read the
same layers and get simpler once step 2 exists.
