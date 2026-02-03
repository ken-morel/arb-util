use super::utils::stringe;
use std::path::PathBuf;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct L10NYaml {
    arb_dir: String,
    template_arb_file: String,
    output_localization_file: String,
}
#[derive(Debug, serde::Deserialize)]
pub struct PubSpec {
    name: String,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub root_dir: PathBuf,
    pub l10n_dir: PathBuf,
    pub arb_template: String,
    pub localizations_file: String,
}

impl Project {
    pub fn load() -> Result<Self, String> {
        let root = stringe("could not get current directory", std::env::current_dir())?;

        let pubspec: PubSpec = stringe(
            "could not parse pubspec.yaml",
            serde_yaml::from_slice(
                stringe(
                    "could not read pubspec.yaml",
                    std::fs::read(root.join("pubspec.yaml")),
                )?
                .as_slice(),
            ),
        )?;

        let config: L10NYaml = stringe(
            "could not parse l10n.yaml",
            serde_yaml::from_slice(
                stringe(
                    "could not read l10n.yaml",
                    std::fs::read(root.join("l10n.yaml")),
                )?
                .as_slice(),
            ),
        )?;
        if !config.arb_dir.starts_with("lib/") {
            return Err(String::from(
                "Please, make sure your configuration arb-dir points to `lib/...`",
            ));
        }

        Ok(Self {
            root_dir: root,
            l10n_dir: match config.arb_dir.strip_suffix("/") {
                // remove possible end slash
                Some(s) => s,
                None => config.arb_dir.as_str(),
            }
            .into(),
            arb_template: config.template_arb_file,
            name: pubspec.name,
            localizations_file: config.output_localization_file,
        })
    }
}
