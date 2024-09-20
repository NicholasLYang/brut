use std::collections::HashSet;
use std::env::current_dir;
use std::fs;
use clean_path::clean;
use camino::{Utf8Path, Utf8PathBuf};
use cargo::core::Workspace;
use cargo::GlobalContext;
use clap::Parser;

mod graph;

fn find_lockfile_dir(path: &Utf8Path) -> Option<Utf8PathBuf> {
    for path in path.ancestors() {
        let lockfile_path = path.join("Cargo.lock");
        if lockfile_path.exists() {
            return Some(lockfile_path.to_path_buf());
        }
    }

    None
}



#[derive(Debug, Parser)]
enum Command {
    Build { files: Vec<Utf8PathBuf> },
}

#[derive(Debug, Parser)]
struct Args {
    /// Currently the path to the workspace manifest. TODO: support inferring
    #[clap(long, global = true)]
    cwd: Option<Utf8PathBuf>,
    #[clap(subcommand)]
    command: Command,
}

fn make_absolute(path: &Utf8Path, cwd: &Utf8Path) -> Utf8PathBuf {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };

    clean(path).try_into().unwrap()
}


fn get_crates_from_files(cwd: &Utf8Path, workspace: &Workspace, files: &[Utf8PathBuf]) -> Result<HashSet<cargo::core::PackageId>, anyhow::Error> {
    let mut crates = HashSet::new();

    for file in files {
        let file = make_absolute(file, cwd);
        println!("{}", file);
        for package in workspace.members() {
            println!("{}", package.root().display());
            if file.starts_with(package.root()) {
                crates.insert(package.package_id());
            }
        }
    }

    Ok(crates)
}


fn main() -> Result<(), anyhow::Error> {
    let ctx = GlobalContext::default()?;

    let args = Args::parse();
    let current_dir: Utf8PathBuf = if let Some(cwd) = args.cwd {
        fs::canonicalize(cwd)?.try_into()?
    } else{
        current_dir()?.try_into()?
    };

    match args.command {
        Command::Build { files } => {
            let ws = Workspace::new(current_dir.as_std_path(), &ctx)?;
            let crates = get_crates_from_files(current_dir.parent().unwrap(), &ws, &files)?;
            println!("{:?}", crates);
        }
    }




    Ok(())
}
