use camino::Utf8PathBuf;
use petgraph::Graph;

struct Crate {
    name: String,
    path: Utf8PathBuf,
}

struct CrateGraph {
    graph: Graph<Crate, ()>
}

