use cargo::core::PackageId;
use cargo_lock::{dependency, Dependency};

pub fn get_dependents_of_pkg(
    tree: &dependency::Tree,
    pkg: PackageId,
) -> Result<Vec<String>, anyhow::Error> {
    let node_idx = tree
        .nodes()
        .get(&Dependency {
            name: pkg.name().to_string().parse()?,
            version: pkg.version().to_string().parse()?,
            source: None,
        })
        .ok_or(anyhow::anyhow!("Package not found"))?;

    Ok(tree
        .graph()
        .neighbors_directed(*node_idx, petgraph::Direction::Incoming)
        .filter_map(|idx| {
            let pkg = &tree.graph()[idx];
            match &pkg.source {
                Some(source) if source.is_path() => Some(tree.graph()[idx].name.to_string()),
                None => Some(tree.graph()[idx].name.to_string()),
                _ => None,
            }
        })
        .collect())
}
