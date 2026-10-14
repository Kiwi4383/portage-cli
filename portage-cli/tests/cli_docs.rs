//! Generate `docs/user/cli/` from what `em`'s derives write.
//!
//! The pages come from the fragments winnow-args' derives write when `em` is
//! compiled with `WINNOW_ARGS_SPEC` set: `just docs` and `just docs-check` set
//! it. Unset, there is nothing to render and the tests that need pages pass
//! without looking.
//!
//! Hidden applets must not appear. Omission is driven by `hide`, not an
//! applet-name allowlist.
//!
//! Refresh committed pages with `just docs`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use portage_cli::cli::Cli;
use winnow_args::Args as _;
use winnow_args_spec::{Catalog, Command, Effect};

const GENERATED_HEADER: &str = "<!-- @generated from em's command line; do not edit -->\n";

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("portage-cli is a workspace member")
        .to_path_buf()
}

fn cli_docs_dir() -> PathBuf {
    workspace_root().join("docs/user/cli")
}

/// `em`'s command line as the derives documented it, when this build wrote
/// the fragments.
fn documented() -> Option<Command> {
    let dir = option_env!("WINNOW_ARGS_SPEC").filter(|dir| !dir.is_empty())?;
    let catalog = Catalog::load(Path::new(dir)).unwrap_or_else(|err| panic!("{err}"));
    Some(
        catalog
            .stitch("Cli")
            .unwrap_or_else(|err| panic!("em's fragments must stitch: {err}")),
    )
}

/// Resolve an href found inside `from_page`'s body to the page-tree-relative
/// key it points at.
fn resolve_href(from_page: &str, href: &str) -> String {
    let path = href.split_once('#').map_or(href, |(p, _)| p);
    let mut parts: Vec<&str> = match from_page.rsplit_once('/') {
        Some((dir, _)) => dir.split('/').filter(|s| !s.is_empty()).collect(),
        None => Vec::new(),
    };
    for seg in path.split('/').filter(|s| !s.is_empty()) {
        match seg {
            "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

/// Every non-anchor, non-external `](TARGET)` link in `body`.
fn markdown_links(body: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(idx) = rest.find("](") {
        rest = &rest[idx + 2..];
        let Some(end) = rest.find(')') else { break };
        let target = &rest[..end];
        rest = &rest[end + 1..];
        if !target.is_empty()
            && !target.starts_with('#')
            && !target.starts_with("http://")
            && !target.starts_with("https://")
        {
            out.push(target);
        }
    }
    out
}

/// One page per visible command: `index.md` for `em`, `query.md`, then
/// `query/belongs.md`.
fn generated_pages(root: &Command) -> BTreeMap<String, String> {
    let mut pages = BTreeMap::new();
    for page in winnow_args_spec::pages(root, "em").unwrap_or_else(|err| panic!("{err}")) {
        let words = &page.path[1..];
        let rel = match words {
            [] => "index.md".to_owned(),
            words => format!("{}.md", words.join("/")),
        };
        // The renderer names a page after its whole path, `em-query-belongs.md`,
        // beside its parent. Here a command's children are in its directory.
        let beside = format!("]({}-", page.stem());
        let below = match words.last() {
            None => "](".to_owned(),
            Some(last) => format!("]({last}/"),
        };
        let md = winnow_args_markdown::render(&page).replace(&beside, &below);
        pages.insert(rel, format!("{GENERATED_HEADER}{}\n", md.trim()));
    }
    pages
}

fn collect_md(dir: &Path) -> BTreeMap<String, String> {
    let mut pages = BTreeMap::new();
    collect_md_into(dir, Path::new(""), &mut pages);
    pages
}

fn collect_md_into(dir: &Path, rel: &Path, pages: &mut BTreeMap<String, String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let entry = entry.expect("read dir entry");
        let path = entry.path();
        let rel_path = rel.join(entry.file_name());
        if path.is_dir() {
            collect_md_into(&path, &rel_path, pages);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let key = rel_path.to_str().expect("utf-8 path").replace('\\', "/");
            pages.insert(key, fs::read_to_string(&path).expect("read markdown"));
        }
    }
}

fn write_pages(dir: &Path, pages: &BTreeMap<String, String>) {
    fs::create_dir_all(dir).expect("create docs/user/cli");
    for (rel, body) in pages {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create command dir");
        }
        fs::write(&path, body).unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
    }
    let committed = collect_md(dir);
    for rel in committed.keys() {
        if !pages.contains_key(rel) {
            fs::remove_file(dir.join(rel))
                .unwrap_or_else(|err| panic!("remove stale {rel}: {err}"));
        }
    }
    remove_empty_dirs(dir);
}

fn remove_empty_dirs(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let children: Vec<_> = entries.map(|e| e.expect("read dir entry").path()).collect();
    for child in &children {
        if child.is_dir() {
            remove_empty_dirs(child);
        }
    }
    if dir != cli_docs_dir()
        && fs::read_dir(dir)
            .map(|mut it| it.next().is_none())
            .unwrap_or(false)
    {
        let _ = fs::remove_dir(dir);
    }
}

fn assert_omits_hidden(pages: &BTreeMap<String, String>) {
    assert!(
        pages.contains_key("index.md"),
        "expected generated index.md"
    );
    assert!(
        pages.contains_key("emerge.md"),
        "expected generated emerge.md"
    );
    assert!(
        pages.contains_key("query.md"),
        "expected generated query.md"
    );
    for (path, body) in pages {
        assert!(
            !path.contains("__helper") && !path.contains("__worker"),
            "hidden applet page leaked: {path}"
        );
        assert!(
            !body.contains("`em __helper") && !body.contains("`em __worker"),
            "hidden applet leaked into {path}"
        );
        assert!(
            !body.contains("](/"),
            "site-root markdown link left in {path}"
        );
    }
}

/// The long names of a command's visible and hidden flags.
fn flag_longs<'a>(longs: impl Iterator<Item = Option<&'a str>>) -> BTreeSet<String> {
    longs.flatten().map(str::to_owned).collect()
}

/// What is documented is what is parsed: the same commands, with the same
/// flags, hidden where the parser hides them.
fn assert_cmd_matches(path: &str, parsed: &winnow_args::help::Command, documented: &Command) {
    let flags = |positional: bool| !positional;
    assert_eq!(
        flag_longs(
            parsed
                .items
                .iter()
                .filter(|item| flags(item.positional))
                .map(|item| item.long)
        ),
        flag_longs(
            documented
                .items
                .iter()
                .filter(|item| flags(item.positional))
                .map(|item| item.long.as_deref())
        ),
        "flag longs mismatch at {path:?}"
    );
    let parsed_subs: Vec<(&str, bool)> = parsed
        .subcommands
        .iter()
        .map(|sub| (sub.name, sub.hide))
        .collect();
    let documented_subs: Vec<(&str, bool)> = documented
        .subcommands
        .iter()
        .map(|sub| (sub.name.as_str(), sub.hide))
        .collect();
    assert_eq!(
        parsed_subs, documented_subs,
        "subcommands mismatch at {path:?}"
    );
    for (sub, child) in parsed.subcommands.iter().zip(&documented.subcommands) {
        let child_path = if path.is_empty() {
            sub.name.to_string()
        } else {
            format!("{path}/{}", sub.name)
        };
        assert_cmd_matches(&child_path, sub.command, &child.command);
    }
}

fn sub<'a>(command: &'a Command, name: &str) -> &'a winnow_args_spec::Sub {
    let found = command.subcommands.iter().find(|sub| sub.name == name);
    found.unwrap_or_else(|| panic!("no `{name}` under `{}`", command.name))
}

#[test]
fn the_documented_command_line_is_the_parsed_one() {
    let Some(root) = documented() else { return };
    assert_eq!(root.name, "em");
    for hidden in ["__helper", "__worker"] {
        assert!(
            sub(&root, hidden).hide,
            "{hidden} must remain documented as hidden (hide, not delete)"
        );
    }
    assert!(!sub(&root, "emerge").hide);
    assert!(!sub(&root, "query").hide);
    assert_cmd_matches("", Cli::HELP, &root);
}

#[test]
fn effects_reach_the_documentation() {
    let Some(root) = documented() else { return };
    let emerge = &sub(&root, "emerge").command;
    assert_eq!(emerge.effect, Some(Effect::Write));
    assert_eq!(
        sub(&root, "depclean").command.effect,
        Some(Effect::Destructive)
    );
    let flag = |command: &Command, long: &str| {
        let found = command
            .items
            .iter()
            .find(|item| item.long.as_deref() == Some(long));
        found.unwrap_or_else(|| panic!("no --{long}")).effect
    };
    assert_eq!(flag(emerge, "depclean"), Some(Effect::Destructive));
    assert_eq!(
        flag(emerge, "pretend"),
        None,
        "--pretend must not lower effect"
    );
}

#[test]
fn generated_markdown_omits_hidden_applets() {
    let Some(root) = documented() else { return };
    let pages = generated_pages(&root);
    assert_omits_hidden(&pages);
    let profile = pages.get("select/profile.md").expect("select/profile.md");
    assert!(
        profile.contains("](profile/list.md)"),
        "nested page must link a child relative to itself, not the tree root:\n{profile}"
    );
    let index = pages.get("index.md").expect("index.md");
    assert!(
        index.contains("](emerge.md)") && index.contains("](query.md)"),
        "index links should be relative to docs/user/cli:\n{index}"
    );
    let query = pages.get("query.md").expect("query.md");
    assert!(query.contains("](query/belongs.md)"), "{query}");
}

/// Structural staleness, not byte-exact rendering. Deliberately does NOT
/// compare rendered bytes — a pure rendering change in winnow-args-markdown
/// isn't a staleness bug and shouldn't fail this; a new/removed applet
/// without a matching regen is. See docs/design/testing.md or ask before
/// tightening this back to an exact-match assertion.
#[test]
fn committed_cli_docs_match_spec() {
    let Some(root) = documented() else { return };
    let pages = generated_pages(&root);
    let dir = cli_docs_dir();
    if std::env::var_os("UPDATE_CLI_DOCS").is_some() {
        write_pages(&dir, &pages);
        return;
    }
    let committed = collect_md(&dir);

    let generated_keys: BTreeSet<&String> = pages.keys().collect();
    let committed_keys: BTreeSet<&String> = committed.keys().collect();
    let missing: Vec<_> = generated_keys.difference(&committed_keys).collect();
    assert!(
        missing.is_empty(),
        "docs/user/cli is missing pages for: {missing:?}. Run `just docs`"
    );
    let orphaned: Vec<_> = committed_keys.difference(&generated_keys).collect();
    assert!(
        orphaned.is_empty(),
        "docs/user/cli has stale pages no longer in the command line: {orphaned:?}. Run `just docs`"
    );

    for (path, body) in &committed {
        for target in markdown_links(body) {
            let resolved = resolve_href(path, target);
            assert!(
                committed.contains_key(&resolved),
                "{path} links to {target:?} (resolved to {resolved:?}), which doesn't exist"
            );
        }
    }
}
