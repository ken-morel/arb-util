use super::utils::stringe;
use std::path::PathBuf;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct L10NYaml {
    arb_dir: String,
    template_arb_file: String,
}

#[derive(Debug, Clone)]
pub struct Project {
    root_dir: PathBuf,
    l10n_dir: PathBuf,
    arb_template: String,
}

impl Project {
    pub fn load() -> Result<Self, String> {
        let root = stringe("could not get current directory", std::env::current_dir())?;
        let l10n = root.join("l10n.yaml");
        if !stringe(
            "could not verify if l10n.yaml exists",
            std::fs::exists(&l10n),
        )? {
            return Err(String::from("l10n.yaml not found make sure you setup your project https://docs.flutter.dev/ui/internationalization"));
        }
        let l10n_raw = stringe("could not read l10n.yaml", std::fs::read(l10n))?;
        let config: L10NYaml = stringe(
            "could not parse l10n.yaml",
            serde_yaml::from_slice(l10n_raw.as_slice()),
        )?;

        Ok(Self {
            root_dir: root,
            l10n_dir: config.arb_dir.into(),
            arb_template: config.template_arb_file,
        })
    }
}
