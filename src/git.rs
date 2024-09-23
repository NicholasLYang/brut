use camino::Utf8PathBuf;
use git2::Repository;
use std::collections::HashSet;

pub fn get_changed_files(
    repo: &Repository,
    base: &str,
    head: Option<String>,
) -> Result<HashSet<Utf8PathBuf>, anyhow::Error> {
    let base = repo.revparse_single(base)?;
    let base_tree = base.peel_to_tree()?;
    let diff = if let Some(head) = head {
        let head = repo.revparse_single(&head)?;
        let head_tree = head.peel_to_tree()?;
        repo.diff_tree_to_tree(Some(&base_tree), Some(&head_tree), None)?
    } else {
        repo.diff_tree_to_workdir_with_index(Some(&base_tree), None)?
    };

    let mut files = HashSet::new();
    for delta in diff.deltas() {
        if delta.status() == git2::Delta::Added || delta.status() == git2::Delta::Modified {
            if let Some(path) = delta.new_file().path() {
                files.insert(path.to_path_buf().try_into()?);
            }
            if let Some(path) = delta.old_file().path() {
                files.insert(path.to_path_buf().try_into()?);
            }
        }
    }

    Ok(files)
}
