use crate::git::get_changed_files;
use crate::graph::get_dependents_of_pkg;
use camino::{Utf8Path, Utf8PathBuf};
use cargo::core::Workspace;
use cargo::GlobalContext;
use cargo_lock::Lockfile;
use clap::Parser;
use clean_path::clean;
use colored::Colorize;
use git2::Repository;
use std::collections::HashSet;
use std::env::current_dir;
use std::fs;
use which::which;

mod git;
mod graph;

#[derive(Debug, Parser)]
enum Command {
    Build {
        #[clap(flatten)]
        args: CommandArgs,
    },
    Check {
        #[clap(flatten)]
        args: CommandArgs,
    },
    Clippy {
        #[clap(flatten)]
        args: CommandArgs,
    },
    Run {
        #[clap(flatten)]
        args: CommandArgs,
    },
    Test {
        #[clap(flatten)]
        args: CommandArgs,
    },
}

#[derive(Debug, Parser, Clone)]
struct CommandArgs {
    /// The base git branch, defaults to `main`
    #[clap(long)]
    base: Option<String>,
    /// The git head, defaults to `HEAD`
    #[clap(long)]
    head: Option<String>,
    #[clap(long)]
    dry_run: bool,
}

#[derive(Debug, Parser)]
struct Args {
    /// Currently the path to the workspace manifest. TODO: support inferring
    #[clap(long, global = true)]
    cwd: Option<Utf8PathBuf>,
    #[clap(subcommand)]
    command: Command,
}

fn find_lockfile_dir(path: &Utf8Path) -> Option<Utf8PathBuf> {
    for path in path.ancestors() {
        if path.join("Cargo.lock").exists() {
            return Some(path.to_path_buf());
        }
    }

    None
}

fn make_absolute(path: &Utf8Path, cwd: &Utf8Path) -> Utf8PathBuf {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };

    clean(path).try_into().unwrap()
}

fn get_crates_from_files(
    cwd: &Utf8Path,
    workspace: &Workspace,
    files: &HashSet<Utf8PathBuf>,
) -> Result<HashSet<cargo::core::PackageId>, anyhow::Error> {
    let mut crates = HashSet::new();

    for file in files {
        let file = make_absolute(file, cwd);
        for package in workspace.members() {
            if file.starts_with(package.root()) {
                crates.insert(package.package_id());
            }
        }
    }

    Ok(crates)
}

fn execute_command(
    workspace_root: &Utf8Path,
    cargo_command: &str,
    dependents: &HashSet<String>,
    dry_run: bool,
) -> Result<(), anyhow::Error> {
    let mut command = std::process::Command::new(which("cargo")?);

    command.arg(cargo_command).current_dir(workspace_root);

    for pkg in dependents {
        command.arg("-p").arg(pkg);
    }

    if dry_run {
        println!(
            "{} {}",
            "Command:".green().bold(),
            format!("{:?}", command).dimmed()
        );
    } else {
        command.spawn()?;
    }

    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    let ctx = GlobalContext::default()?;

    let args = Args::parse();
    let cwd: Utf8PathBuf = if let Some(cwd) = args.cwd {
        fs::canonicalize(cwd)?.try_into()?
    } else {
        current_dir()?.try_into()?
    };

    let (args, command) = match args.command {
        Command::Build { args } => (args, "build"),
        Command::Check { args } => (args, "check"),
        Command::Clippy { args } => (args, "clippy"),
        Command::Run { args } => (args, "run"),
        Command::Test { args } => (args, "test"),
    };

    let base = args.base.as_deref().unwrap_or("main");
    let repo = Repository::discover(&cwd)?;
    let git_root: &Utf8Path = repo
        .workdir()
        .expect("git has working directory")
        .try_into()?;

    let files = get_changed_files(&repo, base, args.head)?;

    let workspace_root = find_lockfile_dir(&cwd).ok_or(anyhow::anyhow!("No Cargo.lock found"))?;

    let ws = Workspace::new(workspace_root.join("Cargo.toml").as_std_path(), &ctx)?;
    // This probably breaks if the workspace root is not the git root
    let pkgs = get_crates_from_files(&git_root, &ws, &files)?;

    let lockfile = Lockfile::load(workspace_root.join("Cargo.lock").as_std_path())?;
    let tree = lockfile.dependency_tree()?;
    let mut dependents = HashSet::new();

    for pkg in pkgs {
        dependents.insert(pkg.name().to_string());
        dependents.extend(get_dependents_of_pkg(&tree, pkg)?);
    }

    if args.dry_run {
        println!(
            "{} {}",
            "Changed files:".green().bold(),
            format!("{:?}", files).dimmed()
        );
        println!(
            "{} {}",
            "Affected packages:".green().bold(),
            format!("{:?}", dependents).dimmed()
        );
    }

    if dependents.is_empty() {
        println!("{}", "No affected packages, skipping".blue().bold());
        return Ok(());
    }

    execute_command(&workspace_root, command, &dependents, args.dry_run)?;

    Ok(())
}
