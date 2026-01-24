use std::{
    fs,
    path::Path,
};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_diagram_extensions")]
    pub diagram_extensions: Vec<String>,

    #[serde(default = "default_openapi_extensions")]
    pub openapi_extensions: Vec<String>,

    #[serde(default = "default_agreements")]
    pub agreements: Vec<String>,

    pub projects: Vec<ProjectConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub name: String,

    #[serde(default)]
    pub c4: C4Config,

    #[serde(default)]
    pub erd: Vec<String>,

    #[serde(default)]
    pub adr: Vec<String>,

    #[serde(default)]
    pub openapi: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct C4Config {
    #[serde(default)]
    pub c1: Vec<String>,
    #[serde(default)]
    pub c2: Vec<String>,
    #[serde(default)]
    pub c3: Vec<String>,
    #[serde(default)]
    pub services: Vec<String>,
}

impl Config {
    pub fn load(explicit_config: Option<&Path>) -> anyhow::Result<Self> {
        let config_path = match explicit_config {
            Some(path) => path.to_path_buf(),
            None => {
                // Default: look for arch-mcp.toml in current working directory
                std::env::current_dir()?
                    .join("arch-mcp.toml")
            }
        };

        let content = fs::read_to_string(&config_path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to read config file '{}': {}",
                config_path.display(),
                e
            )
        })?;

        let mut cfg: Config = toml::from_str(&content).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse config file '{}': {}",
                config_path.display(),
                e
            )
        })?;

        normalize_extensions(&mut cfg.diagram_extensions);
        normalize_extensions(&mut cfg.openapi_extensions);
        normalize_paths(&mut cfg.agreements);

        for project in &mut cfg.projects {
            normalize_paths(&mut project.c4.c1);
            normalize_paths(&mut project.c4.c2);
            normalize_paths(&mut project.c4.c3);
            normalize_paths(&mut project.c4.services);
            normalize_paths(&mut project.erd);
            normalize_paths(&mut project.adr);
            normalize_paths(&mut project.openapi);
        }

        Ok(cfg)
    }
}

fn default_diagram_extensions() -> Vec<String> {
    vec!["mdx".to_string(), "puml".to_string(), "dot".to_string()]
}

fn default_openapi_extensions() -> Vec<String> {
    vec!["yaml".to_string(), "yml".to_string()]
}

fn default_agreements() -> Vec<String> {
    vec!["content/docs/backend".to_string()]
}

fn normalize_extensions(exts: &mut Vec<String>) {
    for ext in exts.iter_mut() {
        let e = ext.trim().trim_start_matches('.').to_ascii_lowercase();
        *ext = e;
    }
    exts.retain(|e| !e.is_empty());
    exts.sort();
    exts.dedup();
}

fn normalize_paths(paths: &mut Vec<String>) {
    for p in paths.iter_mut() {
        let trimmed = p.trim().to_string();
        *p = trimmed;
    }
    paths.retain(|p| !p.is_empty());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_config_with_defaults() {
        let toml_str = r#"
projects = [
  { name = "example-project" }
]
"#;

        let mut cfg: Config = toml::from_str(toml_str).expect("parse config");
        normalize_extensions(&mut cfg.diagram_extensions);
        normalize_extensions(&mut cfg.openapi_extensions);
        normalize_paths(&mut cfg.agreements);

        assert_eq!(cfg.projects.len(), 1);
        assert_eq!(cfg.projects[0].name, "example-project");
        assert_eq!(cfg.diagram_extensions, vec!["dot", "mdx", "puml"]);
        assert_eq!(cfg.openapi_extensions, vec!["yaml", "yml"]);
        assert_eq!(cfg.agreements, vec!["content/docs/backend"]);
    }

    #[test]
    fn parse_full_config_shape() {
        let toml_str = r#"
diagram_extensions = ["puml", ".dot"]
openapi_extensions = ["YAML", "yml"]
agreements = ["content/docs/backend", "content/docs/frontend"]

[[projects]]
name = "example-project"
erd = ["arch/erd"]
adr = ["arch/adr"]
openapi = ["openapi-spec"]

[projects.c4]
c1 = ["arch/c4"]
c2 = ["arch/c4"]
c3 = ["arch/c4"]
services = ["arch/c4/services"]
"#;

        let mut cfg: Config = toml::from_str(toml_str).expect("parse config");
        normalize_extensions(&mut cfg.diagram_extensions);
        normalize_extensions(&mut cfg.openapi_extensions);
        normalize_paths(&mut cfg.agreements);

        assert_eq!(cfg.diagram_extensions, vec!["dot", "puml"]);
        assert_eq!(cfg.openapi_extensions, vec!["yaml", "yml"]);
        assert_eq!(
            cfg.agreements,
            vec!["content/docs/backend", "content/docs/frontend"]
        );

        let p = &cfg.projects[0];
        assert_eq!(p.name, "example-project");
        assert_eq!(p.c4.c1, vec!["arch/c4"]);
        assert_eq!(p.c4.services, vec!["arch/c4/services"]);
        assert_eq!(p.erd, vec!["arch/erd"]);
        assert_eq!(p.adr, vec!["arch/adr"]);
        assert_eq!(p.openapi, vec!["openapi-spec"]);
    }
}
