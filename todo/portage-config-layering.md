# One resolved Portage configuration, instead of one per consumer

Status: 🟡 steps 1–5 done 2026-10-22/23; step 6 partly (targeted seeding), the rest waits on [[features-parity]]. Started from `em portageq envvar` not
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

1. ✅ **ACCEPT_LICENSE/PROPERTIES/RESTRICT** (2026-10-22): stacked by Portage's rule (concat, cut
   at the last `*`/`-*`, negations kept) with their own merge in `merge_use_var`, the
   environment as the top layer, `* -@EULA` / `*` when empty (`fb42cbda`, `446ea895`), and
   `AcceptSet` applying tokens in order, which also fixed package.license overrides (`a582eaf2`).
   Checked live: geekbench refused, accepted via `ACCEPT_LICENSE=geekbench` and package.license,
   as emerge does. ACCEPT_PROPERTIES/RESTRICT still print empty where portageq prints
   make.globals' `*` (the resolver already defaults to `*`): that is step 2.
2. ✅ (2026-10-23, `85714279`) **A `Settings` value** (see "Settings: home and lifecycle"). Typed lookup, the incremental
   and ACCEPT_* rules above, the USE_EXPAND rebuild from the USE the profile code already
   resolves correctly. Each value keeps which layer it came from (for `--info -v` and
   debugging). Test: an ignored live test diffing `portageq envvar -v` for every name, in a
   clean environment and with `CFLAGS=…`/`FEATURES=…` set, plus `PORTAGE_CONFIGROOT` scratch
   roots for the `-*`/`-x`/profile-only cases. The derived ROOT/EROOT/… are not compared
   here; step 3 adds them.
3. ✅ **`portageq envvar` and `--info` read `Settings`** (`fa5fe890`, `f5e934f3`). All 176 variables em prints
   match portageq; missing are Portage internals, PORTAGE_REPOSITORIES, PORTAGE_ARCHLIST. `--info`'s 21 variables
   match `emerge --info`. Known: `ARCH=x` in the environment gives portageq `''`, em the profile's value; a scratch
   root's unset GRUB_PLATFORMS is `''` in portageq, absent in em.
4. ✅ **The resolver reads `Settings`** (`5cdfa855`): fixed `ACCEPT_KEYWORDS` from the environment being ignored.
   env.d and make.globals are not layered there (nothing a resolve reads). Parity table and resolve-mode matrix
   unchanged.
5. ✅ **Hand-assembled readers retired** (`89f17f5c`): `em etc`, the mirror and elog lookups, mirrordist. Fixed a
   merge diverting env.d-masked files to `._cfg` (checked in a sandbox with sys-apps/sandbox). Kept: the PKGDIR chain,
   which ignores make.globals for a non-host root on purpose; `privilege.rs`'s DISTDIR (runs before the runtime);
   `read_make_conf_var_for_roots` (make.conf-only CFLAGS etc. for per-entry roots); `ProfileStack` cannot be empty,
   so a root without a profile has no `Settings`.
6. 🟡 **The build environment:** done for CONFIG_PROTECT(_MASK), GENTOO_MIRRORS, PORTAGE_ELOG_CLASSES/SYSTEM, seeded
   from `Settings`. Seeding everything is not safe yet: make.globals' FETCHCOMMAND would switch em's fetcher to wget,
   PORTAGE_COMPRESS and others change behaviour, and FEATURES needs the [[features-parity]] table (D3) first.

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
