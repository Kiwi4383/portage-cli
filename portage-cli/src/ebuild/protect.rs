//! `CONFIG_PROTECT` / `CONFIG_PROTECT_MASK` resolution and `._cfg` diversion
//!
//! The lists are Portage's stacked values: a build shell holds them seeded from the
//! settings (make.globals and env.d included), and a caller without one reads the settings.

use camino::{Utf8Path, Utf8PathBuf};

/// CONFIG_PROTECT / CONFIG_PROTECT_MASK resolution (portage's `ConfigProtect`)
///
/// A path is protected when the longest matching `CONFIG_PROTECT` prefix is
/// longer than the longest matching `CONFIG_PROTECT_MASK` prefix. Protected
/// files that already exist and differ are diverted to `._cfgNNNN_<name>`
/// for `dispatch-conf`/`etc-update` instead of being overwritten.
///
/// Deliberately real-Portage semantics, not the PMS 13.3.3 letter (which
/// gives an ancestor `CONFIG_PROTECT_MASK` unconditional priority regardless
/// of how deep the matching `CONFIG_PROTECT` entry is). Matching what real
/// `emerge` actually does on a system is the intended behavior here.
pub(crate) struct ConfigProtect {
    protect: Vec<String>,
    mask: Vec<String>,
}

impl ConfigProtect {
    /// Read the lists from a build shell
    pub(crate) fn from_shell(shell: &portage_repo::EbuildShell) -> Self {
        let read = |name: &str| shell.get_var(name).unwrap_or_default();
        Self::from_lists(&read("CONFIG_PROTECT"), &read("CONFIG_PROTECT_MASK"))
    }

    /// Read the lists from a config root's settings, for callers with no build shell (`em etc`)
    pub(crate) fn from_settings(settings: &portage_repo::Settings) -> Self {
        let read = |name: &str| settings.get(name).unwrap_or_default();
        Self::from_lists(read("CONFIG_PROTECT"), read("CONFIG_PROTECT_MASK"))
    }

    /// `/etc` is always protected, as make.globals guarantees.
    fn from_lists(protect: &str, mask: &str) -> Self {
        let paths = |list: &str| -> Vec<String> {
            list.split_whitespace()
                .map(|s| s.trim_end_matches('/').to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };
        let mut protect = paths(protect);
        if !protect.iter().any(|p| p == "/etc") {
            protect.push("/etc".to_string());
        }
        Self {
            protect,
            mask: paths(mask),
        }
    }

    /// The protected directories themselves, for a caller that has to walk
    /// them rather than test a single path.
    pub(crate) fn protected_dirs(&self) -> &[String] {
        &self.protect
    }

    /// Length of the longest entry in `list` that prefix-matches `obj` on
    /// whole components (`obj == p` or `obj` under `p/`); 0 if none.
    fn longest_match(list: &[String], obj: &str) -> usize {
        list.iter()
            .filter(|p| obj == p.as_str() || obj.starts_with(&format!("{p}/")))
            .map(String::len)
            .max()
            .unwrap_or(0)
    }

    pub(crate) fn is_protected(&self, obj: &Utf8Path) -> bool {
        let obj = obj.as_str();
        Self::longest_match(&self.protect, obj) > Self::longest_match(&self.mask, obj)
    }

    /// Explicit lists, for a test that needs a specific protect/mask pair
    /// without a shell or a `make.conf`.
    #[cfg(test)]
    pub(crate) fn for_test(protect: &[&str], mask: &[&str]) -> Self {
        Self {
            protect: protect.iter().map(|s| (*s).to_string()).collect(),
            mask: mask.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    /// A config-protection set that protects nothing (for tests / contexts
    /// where protection does not apply).
    #[cfg(test)]
    pub(crate) fn none() -> Self {
        Self {
            protect: vec![],
            mask: vec![],
        }
    }
}

/// portage's `new_protect_filename`: the next `._cfgNNNN_<name>` beside
/// `dest` (highest existing index + 1), plus the most recent existing one
/// so the caller can reuse it when the content already matches.
pub(crate) fn scan_cfg(dest: &Utf8Path) -> (Utf8PathBuf, Option<Utf8PathBuf>) {
    let dir = dest.parent().unwrap_or_else(|| Utf8Path::new("/"));
    let name = dest.file_name().unwrap_or_default();
    let mut highest: i32 = -1;
    let mut latest: Option<Utf8PathBuf> = None;
    if let Ok(rd) = std::fs::read_dir(dir.as_std_path()) {
        for entry in rd.flatten() {
            let Ok(f) = entry.file_name().into_string() else {
                continue;
            };
            // ._cfg<4 digits>_<name>
            let Some(rest) = f.strip_prefix("._cfg") else {
                continue;
            };
            if rest.len() > 5
                && rest.as_bytes()[4] == b'_'
                && &rest[5..] == name
                && let Ok(n) = rest[..4].parse::<i32>()
                && n > highest
            {
                highest = n;
                latest = Some(dir.join(&f));
            }
        }
    }
    (dir.join(format!("._cfg{:04}_{name}", highest + 1)), latest)
}
