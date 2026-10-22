//! The [`Settings`] of the config root a command selects, and the variables Portage derives
//! from the roots and repositories rather than from any config file

use anyhow::{Context, Result};
use camino::Utf8Path;
use portage_repo::{ConfSource, Environment, ProfileStack, Settings, SettingsSources};
use portage_resolve::Roots;

use crate::cli::Cli;

/// Portage's defaults for a host install; a prefix has none of its own yet
const MAKE_GLOBALS: &str = "/usr/share/portage/config/make.globals";

/// The profile stack of `config_root`, its user profiles included; `None` without a `make.profile`
pub(crate) fn profile_stack(
    config_root: &Utf8Path,
    config_overlay: Option<&Utf8Path>,
) -> Result<Option<ProfileStack>> {
    let Ok(profile_path) =
        std::fs::canonicalize(config_root.join("etc/portage/make.profile").as_std_path())
    else {
        return Ok(None);
    };
    let mut stack = ProfileStack::build_in(profile_path, config_root)
        .context("building profile stack")?
        .with_user_profile(config_root.join("etc/portage/profile").into_std_path_buf())
        .context("loading the user profile")?;
    // `--prefix`'s config overlay (`<prefix>/etc/portage/profile`) layers on
    // the host profile, same as its `package.use`/`bashrc`. Site
    // `use.force`/`use.mask` there have to reach ebuild phases.
    if let Some(overlay) = config_overlay {
        stack = stack
            .with_user_profile(overlay.join("profile").into_std_path_buf())
            .context("loading the prefix overlay profile")?;
    }
    Ok(Some(stack))
}

/// make.conf(5) files of `config_root`: each a file or a Flat directory of fragments, the legacy
/// `etc/make.conf` first so `etc/portage/make.conf` overrides it, as in Portage
pub(crate) fn make_conf_files(config_root: &Utf8Path) -> Result<Vec<std::path::PathBuf>> {
    portage_repo::expand_make_conf_paths([
        config_root.join("etc/make.conf"),
        config_root.join("etc/portage/make.conf"),
    ])
    .context("listing make.conf")
}

/// The settings of the config root `cli` selects; `None` when it has no profile
pub(crate) async fn load(cli: &Cli) -> Result<Option<Settings>> {
    let roots = cli.roots();
    let config_root = roots.config().unwrap_or(Utf8Path::new("/"));
    let Some(stack) = profile_stack(config_root, roots.config_overlay())? else {
        return Ok(None);
    };
    let confs = make_conf_files(config_root)?;
    let confs: Vec<ConfSource> = confs.iter().map(|p| ConfSource::File(p)).collect();
    let env_d = roots.merge_root().join("etc/profile.env");
    let globals = Utf8Path::new(MAKE_GLOBALS);
    let env = Environment::process();
    let repo = crate::crossdev::main_repo(cli)?;
    let mut shell = repo.shell().await.context("creating shell")?;
    let settings = Settings::evaluate(
        &mut shell,
        SettingsSources {
            stack: &stack,
            env_d: env_d
                .is_file()
                .then(|| ConfSource::File(env_d.as_std_path())),
            globals: (roots.eprefix().is_none() && globals.is_file())
                .then(|| ConfSource::File(globals.as_std_path())),
            confs: &confs,
            env: &env,
        },
    )
    .await
    .context("evaluating the configuration")?;
    Ok(Some(settings))
}

/// The variables Portage derives instead of reading: the roots, and the repositories as
/// PORTDIR/PORTDIR_OVERLAY
pub(crate) fn derived(roots: &Roots) -> Result<Vec<(&'static str, String)>> {
    let slash = |path: &Utf8Path| {
        let path = path.as_str().trim_end_matches('/');
        format!("{path}/")
    };
    let eprefix = roots.eprefix().map_or(String::new(), |p| {
        p.as_str().trim_end_matches('/').to_owned()
    });
    let merge_root = roots.merge_root().as_str().trim_end_matches('/');
    let root = merge_root
        .strip_suffix(eprefix.as_str())
        .unwrap_or(merge_root);
    let root = slash(Utf8Path::new(root));
    let sysroot = roots.build_sysroot().map_or_else(|| "/".to_owned(), slash);
    let broot = roots
        .broot()
        .map(|p| p.as_str().trim_end_matches('/').to_owned())
        .unwrap_or_default();
    let repos = roots.repos_conf().context("reading repos.conf")?;
    let (portdir, overlays) = repo_locations(&repos);
    let mut vars = vec![
        ("ROOT", root.clone()),
        ("EPREFIX", eprefix.clone()),
        (
            "EROOT",
            slash(Utf8Path::new(&format!(
                "{}{eprefix}",
                root.trim_end_matches('/')
            ))),
        ),
        ("SYSROOT", sysroot.clone()),
        (
            "ESYSROOT",
            slash(Utf8Path::new(&format!(
                "{}{eprefix}",
                sysroot.trim_end_matches('/')
            ))),
        ),
        ("BROOT", broot),
        (
            "PORTAGE_CONFIGROOT",
            slash(roots.config().unwrap_or(Utf8Path::new("/"))),
        ),
        ("PORTDIR_OVERLAY", overlays),
    ];
    vars.extend(portdir.map(|p| ("PORTDIR", p)));
    Ok(vars)
}

/// `PORTDIR` and `PORTDIR_OVERLAY` as Portage derives them: the main repo's location, and every
/// other repo's, space-separated
fn repo_locations(repos: &portage_repo::ReposConf) -> (Option<String>, String) {
    // Portage's own repos.conf names `gentoo` the main repo when a config root does not.
    let main_repo = repos.main_repo().or_else(|| repos.find("gentoo"));
    let main = main_repo.map(|r| r.name);
    let location =
        |r: &portage_repo::RepoEntry| r.location.as_path().map(|p| p.display().to_string());
    let overlays: Vec<String> = repos
        .repos()
        .iter()
        .filter(|r| Some(r.name) != main)
        .filter_map(location)
        .collect();
    (main_repo.and_then(location), overlays.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portdir_is_the_main_repo_and_portdir_overlay_the_rest() {
        let root = tempfile::tempdir().unwrap();
        let conf = root.path().join("etc/portage/repos.conf");
        std::fs::create_dir_all(&conf).unwrap();
        std::fs::write(
            conf.join("repos.conf"),
            "[DEFAULT]\nmain-repo = gentoo\n\n[gentoo]\nlocation = /r/gentoo\n\n\
             [guru]\nlocation = /r/guru\n\n[local]\nlocation = /r/local\n",
        )
        .unwrap();
        let repos = portage_repo::ReposConf::load_rooted(
            camino::Utf8Path::from_path(root.path()).unwrap(),
            &[],
        )
        .unwrap();
        let (portdir, overlays) = repo_locations(&repos);
        assert_eq!(portdir.as_deref(), Some("/r/gentoo"));
        let mut overlays: Vec<&str> = overlays.split(' ').collect();
        overlays.sort();
        assert_eq!(overlays, ["/r/guru", "/r/local"]);
    }
}
