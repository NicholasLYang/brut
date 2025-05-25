use cargo::core::Workspace;
use cargo_lock::dependency::Tree;
use cargo_lock::{Checksum, Dependency, Lockfile, SourceId, Version};
use git2::Repository;
use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;

pub fn get_changed_packages_from_lockfile(
    repo: &Repository,
    ws: &Workspace,
    head_lockfile: &Lockfile,
    base: &str,
) -> Result<HashSet<String>, anyhow::Error> {
    let base_lockfile = load_lockfile(repo, base)?;

    let mut changed_packages = HashSet::new();
    for pkg in ws.members() {
        if pkg.package_id().source_id().is_path()
            && package_has_changed(
                base_lockfile.dependency_tree()?,
                head_lockfile.dependency_tree()?,
                &Dependency {
                    name: pkg.name().to_string().parse()?,
                    version: pkg.version().clone(),
                    source: None,
                },
            )?
        {
            changed_packages.insert(pkg.name().to_string());
        }
    }

    Ok(changed_packages)
}

/// Loads lockfile at the given git reference
fn load_lockfile(repo: &Repository, git_ref: &str) -> Result<Lockfile, anyhow::Error> {
    let lockfile_object = repo
        .revparse_single(git_ref)?
        .peel_to_tree()?
        .get_path(Path::new("Cargo.lock"))?
        .to_object(repo)?;

    // We assume that the lockfile is in the root of the repo
    let lockfile_blob = lockfile_object
        .as_blob()
        .ok_or(anyhow::anyhow!("Lockfile blob not found"))?;

    let lockfile_str = std::str::from_utf8(lockfile_blob.content())?;

    Ok(Lockfile::from_str(lockfile_str)?)
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct LockfileEntry {
    name: String,
    version: Version,
    source: Option<SourceId>,
    checksum: Option<Checksum>,
}

/// Compares a package's dependencies between two lockfiles
fn package_has_changed(
    old_lockfile: Tree,
    new_lockfile: Tree,
    pkg: &Dependency,
) -> Result<bool, anyhow::Error> {
    // If the package is not in the new lockfile, we ignore it (we assume the Cargo.toml has changed)
    let Some(new_pkg_idx) = new_lockfile.nodes().get(&pkg) else {
        return Ok(false);
    };
    let Some(old_pkg_idx) = old_lockfile.nodes().get(&pkg) else {
        return Ok(false);
    };

    let lockfile_deps: Vec<_> = old_lockfile
        .graph()
        .neighbors_directed(*old_pkg_idx, petgraph::Direction::Outgoing)
        .map(|idx| LockfileEntry {
            name: old_lockfile.graph()[idx].name.to_string(),
            version: old_lockfile.graph()[idx].version.clone(),
            source: old_lockfile.graph()[idx].source.clone(),
            checksum: old_lockfile.graph()[idx].checksum.clone(),
        })
        .collect();

    let new_lockfile_deps = new_lockfile
        .graph()
        .neighbors_directed(*new_pkg_idx, petgraph::Direction::Outgoing)
        .map(|idx| LockfileEntry {
            name: new_lockfile.graph()[idx].name.to_string(),
            version: new_lockfile.graph()[idx].version.clone(),
            source: new_lockfile.graph()[idx].source.clone(),
            checksum: new_lockfile.graph()[idx].checksum.clone(),
        })
        .collect::<HashSet<_>>();

    if lockfile_deps.len() != new_lockfile_deps.len() {
        return Ok(true);
    }

    for dep in lockfile_deps {
        if !new_lockfile_deps.contains(&dep) {
            return Ok(true);
        }
    }

    Ok(false)
}
