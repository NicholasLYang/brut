use anyhow::anyhow;
use camino::Utf8Path;
use cargo::core::Workspace;
use globset::Glob;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    global_dependencies: Vec<String>,
}

impl Config {
    fn load_from_file(dir: &Utf8Path) -> Result<Self, anyhow::Error> {
        let path = dir.join("brut.toml");
        let contents = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }

    fn load_from_workspace(workspace: &Workspace) -> Result<Self, anyhow::Error> {
        let metadata = workspace.custom_metadata().ok_or(anyhow!("no metadata"))?;

        Ok(metadata
            .get("brut")
            .ok_or(anyhow!("no brut metadata"))?
            .as_table()
            .ok_or(anyhow!("brut metadata is not a table"))?
            .get("config")
            .ok_or(anyhow!("no config"))?
            .as_table()
            .ok_or(anyhow!("config is not a table"))?
            .clone()
            .try_into()?)
    }

    pub fn global_deps_matcher(&self) -> Result<globset::GlobSet, anyhow::Error> {
        let mut builder = globset::GlobSetBuilder::new();
        for dep in &self.global_dependencies {
            builder.add(Glob::new(dep)?);
        }

        Ok(builder.build()?)
    }

    pub fn load(dir: &Utf8Path, workspace: &Workspace) -> Result<Option<Self>, anyhow::Error> {
        let config_from_file = Self::load_from_file(dir);
        let config_from_workspace = Self::load_from_workspace(workspace);
        match (config_from_file, config_from_workspace) {
            (Ok(_), Ok(_)) => Err(anyhow!(
                "Both brut.toml and workspace metadata found, only one is allowed"
            )),
            (Ok(config), _) => Ok(Some(config)),
            (_, Ok(config)) => Ok(Some(config)),
            (Err(_), Err(_)) => Ok(None),
        }
    }
}
