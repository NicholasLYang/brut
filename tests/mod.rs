use assert_cmd::Command;
use std::fs;

fn create_repo(fixture_name: &str) -> Result<tempfile::TempDir, anyhow::Error> {
    let fixture_path = format!("fixtures/{}", fixture_name);
    let dir = tempfile::tempdir()?;
    println!("{}", fs::canonicalize(dir.path())?.display());
    fs::copy(fixture_path, dir.path())?;
    git2::Repository::init(dir.path())?;

    Ok(dir)
}

#[test]
fn test_basic() -> Result<(), anyhow::Error> {
    let dir = create_repo("basic")?;

    let mut cmd = Command::cargo_bin("brut")?;
    cmd.arg("build")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Changed files:"));

    Ok(())
}
