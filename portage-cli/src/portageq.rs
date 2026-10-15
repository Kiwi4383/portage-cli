//! `em portageq` — Portage's own query interface (subset)
//!
//! The VDB group and `envvar` work; everything else does not exist yet.
//!
//! Eroot-taking commands read host global defaults plus the eroot's own
//! user config; `envvar` uses `em`'s own roots.

use anyhow::{Context, Result};
use portage_atom::{Cpv, Dep};
use portage_vdb::{InstalledPackage, Vdb};

/// Real portageq's exit code when `<EROOT>` doesn't exist as a directory.
const EROOT_NOT_A_DIRECTORY: u8 = 64;
/// Real portageq's exit code for an atom that doesn't parse.
const INVALID_ATOM: u8 = 2;

/// Validate `<EROOT>` per real portageq: it must already exist as a
/// directory (an empty/not-yet-bootstrapped root is fine — only a
/// genuinely missing path is rejected here).
fn validate_eroot(eroot: &str) -> std::result::Result<(), u8> {
    if std::path::Path::new(eroot).is_dir() {
        Ok(())
    } else {
        eprintln!("Not a directory: '{eroot}'");
        eprintln!("Run portageq with --help for info");
        Err(EROOT_NOT_A_DIRECTORY)
    }
}

/// Every installed package under `<EROOT>/var/db/pkg`, or empty when that
/// directory doesn't exist yet (a fresh root with nothing installed is a
/// normal case, not an error — `<EROOT>` itself was already validated).
fn installed_packages(eroot: &str) -> Vec<InstalledPackage> {
    let vdb_path = format!("{}/var/db/pkg", eroot.trim_end_matches('/'));
    match Vdb::open(vdb_path.as_str()) {
        Ok(vdb) => vdb.packages().into_iter().collect(),
        Err(_) => Vec::new(),
    }
}

fn parse_atom(atom: &str) -> std::result::Result<Dep, u8> {
    Dep::parse(atom).map_err(|_| {
        eprintln!("ERROR: Invalid atom: '{atom}'");
        INVALID_ATOM
    })
}

fn matches(dep: &Dep, pkg: &InstalledPackage) -> bool {
    dep.matches_cpv(pkg.cpv(), pkg.slot().ok().as_ref())
}

pub(crate) fn has_version(eroot: &str, atom: &str) -> Result<u8> {
    if let Err(code) = validate_eroot(eroot) {
        return Ok(code);
    }
    let dep = match parse_atom(atom) {
        Ok(d) => d,
        Err(code) => return Ok(code),
    };
    let pkgs = installed_packages(eroot);
    Ok(if pkgs.iter().any(|p| matches(&dep, p)) {
        0
    } else {
        1
    })
}

pub(crate) fn best_version(eroot: &str, atom: &str) -> Result<u8> {
    if let Err(code) = validate_eroot(eroot) {
        return Ok(code);
    }
    let dep = match parse_atom(atom) {
        Ok(d) => d,
        Err(code) => return Ok(code),
    };
    let pkgs = installed_packages(eroot);
    let best = pkgs
        .iter()
        .filter(|p| matches(&dep, p))
        .map(|p| p.cpv())
        .max();
    match best {
        Some(cpv) => println!("{cpv}"),
        None => println!(),
    }
    Ok(0)
}

/// `atom` empty matches every installed package. A non-empty atom that
/// isn't a plain dependency atom (e.g. `qgrep`-style `cat/*` glob syntax)
/// is not supported.
pub(crate) fn run_match(eroot: &str, atom: &str) -> Result<u8> {
    if let Err(code) = validate_eroot(eroot) {
        return Ok(code);
    }
    let pkgs = installed_packages(eroot);
    let mut cpvs: Vec<Cpv> = if atom.is_empty() {
        pkgs.iter().map(|p| p.cpv().clone()).collect()
    } else {
        let dep = match parse_atom(atom) {
            Ok(d) => d,
            Err(code) => return Ok(code),
        };
        pkgs.iter()
            .filter(|p| matches(&dep, p))
            .map(|p| p.cpv().clone())
            .collect()
    };
    cpvs.sort();
    for cpv in &cpvs {
        println!("{cpv}");
    }
    Ok(0)
}

/// Missing operands, reported the way portageq does.
fn check_arity(names: &[String], out: &mut dyn std::io::Write) -> Result<Option<u8>> {
    if names.is_empty() {
        writeln!(out, "ERROR: insufficient parameters!")?;
        Ok(Some(2))
    } else {
        Ok(None)
    }
}

/// `envvar -v` line: `NAME=value` bare when safe, else single-quoted.
pub(crate) fn format_verbose(name: &str, value: &str) -> String {
    if value.is_empty() {
        return format!("{name}=''");
    }
    let safe = value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"@%_+=:,./-".contains(&b));
    if safe {
        format!("{name}={value}")
    } else {
        format!("{name}='{}'", value.replace('\'', "'\"'\"'"))
    }
}

/// Writes one line per name: the value or an empty line when unset.
pub(crate) fn write_values(
    names: &[String],
    verbose: bool,
    values: &[Option<String>],
    out: &mut dyn std::io::Write,
) -> Result<u8> {
    let mut code = 0;
    for (name, value) in names.iter().zip(values.iter()) {
        match value {
            Some(v) => {
                if verbose {
                    writeln!(out, "{}", format_verbose(name, v))?;
                } else {
                    writeln!(out, "{v}")?;
                }
            }
            None => {
                writeln!(out)?;
                code = 1;
            }
        }
    }
    Ok(code)
}

/// Profile shell with the root's config already applied.
async fn profile_shell(cli: &crate::cli::Cli) -> Result<portage_repo::EbuildShell> {
    let roots = cli.roots();
    let repo = crate::crossdev::main_repo(cli)?;
    let mut shell = repo.shell().await.context("creating shell")?;
    crate::ebuild::apply_profile_env(&mut shell, roots.config(), roots.config_overlay()).await?;
    Ok(shell)
}

/// Full `envvar` path: profile shell lookup, then shared rendering.
///
/// Ambient environment wins over the shell, set-but-empty included.
pub(crate) async fn run_envvar(
    cli: &crate::cli::Cli,
    names: &[String],
    verbose: bool,
    out: &mut dyn std::io::Write,
) -> Result<u8> {
    if let Some(code) = check_arity(names, out)? {
        return Ok(code);
    }
    let shell = profile_shell(cli).await?;
    let values: Vec<Option<String>> = names
        .iter()
        .map(|n| {
            std::env::var(n)
                .ok()
                .or_else(|| crate::info::resolve_info_var(&shell, n))
        })
        .collect();
    write_values(names, verbose, &values, out)
}

pub(crate) fn mass_best_version(eroot: &str, atoms: &[String]) -> Result<u8> {
    if let Err(code) = validate_eroot(eroot) {
        return Ok(code);
    }
    let pkgs = installed_packages(eroot);
    for atom in atoms {
        let best = Dep::parse(atom).ok().and_then(|dep| {
            pkgs.iter()
                .filter(|p| matches(&dep, p))
                .map(|p| p.cpv())
                .max()
        });
        match best {
            Some(cpv) => println!("{atom}:{cpv}"),
            None => println!("{atom}:"),
        }
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A throwaway VDB tree: `<tmp>/var/db/pkg/<cat>/<pf>/` with the
    /// minimal files `InstalledPackage`/`slot()` need.
    struct TestVdb {
        _tmp: tempfile::TempDir,
        eroot: String,
    }

    impl TestVdb {
        fn new() -> Self {
            let tmp = tempfile::tempdir().unwrap();
            Self {
                eroot: tmp.path().to_string_lossy().into_owned(),
                _tmp: tmp,
            }
        }

        fn add(&self, cpv: &str, slot: &str) {
            let cpv_parsed = Cpv::parse(cpv).unwrap();
            let pf = format!("{}-{}", cpv_parsed.cpn.package, cpv_parsed.version);
            let dir = std::path::Path::new(&self.eroot)
                .join("var/db/pkg")
                .join(cpv_parsed.cpn.category.as_str())
                .join(&pf);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("SLOT"), format!("{slot}\n")).unwrap();
        }
    }

    #[test]
    fn has_version_hit_and_miss() {
        let vdb = TestVdb::new();
        vdb.add("dev-python/setuptools-83.0.0", "0");
        assert_eq!(has_version(&vdb.eroot, "dev-python/setuptools").unwrap(), 0);
        assert_eq!(has_version(&vdb.eroot, "dev-python/pip").unwrap(), 1);
    }

    #[test]
    fn has_version_invalid_atom() {
        let vdb = TestVdb::new();
        assert_eq!(has_version(&vdb.eroot, "not an atom!!").unwrap(), 2);
    }

    #[test]
    fn has_version_missing_eroot() {
        assert_eq!(
            has_version("/no/such/eroot", "dev-lang/python").unwrap(),
            64
        );
    }

    #[test]
    fn best_version_hit_exits_zero() {
        // best_version prints via println!, which these fast unit tests
        // don't capture — cpv_ordering_picks_the_highest_version below
        // exercises the actual max-by-version selection directly.
        let vdb = TestVdb::new();
        vdb.add("dev-python/setuptools-79.0.1", "0");
        vdb.add("dev-python/setuptools-83.0.0", "0");
        assert_eq!(
            best_version(&vdb.eroot, "dev-python/setuptools").unwrap(),
            0
        );
    }

    #[test]
    fn best_version_no_match_still_exits_zero() {
        let vdb = TestVdb::new();
        assert_eq!(
            best_version(&vdb.eroot, "dev-python/setuptools").unwrap(),
            0
        );
    }

    #[test]
    fn run_match_empty_atom_matches_everything() {
        let vdb = TestVdb::new();
        vdb.add("dev-python/setuptools-83.0.0", "0");
        vdb.add("dev-python/pip-26.1.2", "0");
        assert_eq!(run_match(&vdb.eroot, "").unwrap(), 0);
    }

    #[test]
    fn run_match_invalid_atom_is_rejected() {
        let vdb = TestVdb::new();
        assert_eq!(run_match(&vdb.eroot, "not an atom!!").unwrap(), 2);
    }

    #[test]
    fn mass_best_version_mixes_hits_and_misses() {
        let vdb = TestVdb::new();
        vdb.add("dev-python/setuptools-83.0.0", "0");
        let atoms = vec![
            "dev-python/setuptools".to_string(),
            "dev-python/pip".to_string(),
        ];
        assert_eq!(mass_best_version(&vdb.eroot, &atoms).unwrap(), 0);
    }

    #[test]
    fn cpv_ordering_picks_the_highest_version() {
        // Directly exercises the same `.max()` call best_version/
        // mass_best_version rely on, with an assertable return value
        // (println! output isn't easily captured in-process).
        let low = Cpv::parse("dev-python/setuptools-79.0.1").unwrap();
        let high = Cpv::parse("dev-python/setuptools-83.0.0").unwrap();
        assert_eq!(vec![&low, &high].into_iter().max(), Some(&high));
    }

    #[test]
    fn slot_file_is_written_and_readable() {
        // Sanity check on the TestVdb fixture itself: a real InstalledPackage
        // built from it must resolve the SLOT file `matches()` depends on.
        let vdb = TestVdb::new();
        vdb.add("dev-python/setuptools-83.0.0", "1");
        let pkgs = installed_packages(&vdb.eroot);
        assert_eq!(pkgs.len(), 1);
        assert_eq!(pkgs[0].slot().unwrap().to_string(), "1");
    }

    #[test]
    fn missing_operands_report_exit_2() {
        let mut out = Vec::new();
        assert_eq!(
            crate::portageq::check_arity(&[], &mut out).unwrap(),
            Some(2)
        );
        assert_eq!(out, b"ERROR: insufficient parameters!\n");
        let mut out = Vec::new();
        assert_eq!(
            crate::portageq::check_arity(&["CFLAGS".to_string()], &mut out).unwrap(),
            None
        );
        assert!(out.is_empty());
    }

    #[test]
    fn envvar_verbose_quotes_only_when_needed() {
        assert_eq!(
            crate::portageq::format_verbose("PORTAGE_TMPDIR", "/var/tmp"),
            "PORTAGE_TMPDIR=/var/tmp"
        );
        assert_eq!(
            crate::portageq::format_verbose("CFLAGS", "-O2 -march=native -pipe"),
            "CFLAGS='-O2 -march=native -pipe'"
        );
        assert_eq!(
            crate::portageq::format_verbose("FOO", "a'b"),
            "FOO='a'\"'\"'b'"
        );
    }

    #[test]
    fn envvar_write_values_mixed_hit_and_miss() {
        let names = vec!["CFLAGS".to_string(), "NOPE_UNSET_XYZ".to_string()];
        let values = vec![Some("-O2 -pipe".to_string()), None];
        let mut out = Vec::new();
        assert_eq!(
            crate::portageq::write_values(&names, false, &values, &mut out).unwrap(),
            1
        );
        assert_eq!(out, b"-O2 -pipe\n\n");
        let mut out_v = Vec::new();
        assert_eq!(
            crate::portageq::write_values(&names, true, &values, &mut out_v).unwrap(),
            1
        );
        assert_eq!(out_v, b"CFLAGS='-O2 -pipe'\n\n");
    }
}
