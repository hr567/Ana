use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub struct Language {
    language_path: PathBuf,
}

impl Language {
    const LANG_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/lang");

    pub fn new<S: AsRef<OsStr>>(lang: S) -> Option<Language> {
        let language_path = Path::new(Self::LANG_DIR).join(lang.as_ref());
        if language_path.exists() {
            Some(Language { language_path })
        } else {
            None
        }
    }

    pub fn from_ext<S: AsRef<OsStr>>(ext: S) -> Option<Language> {
        let mut language_path = Path::new(Self::LANG_DIR).join(ext.as_ref());
        if language_path.set_extension("default") && language_path.exists() {
            Some(Language { language_path })
        } else {
            None
        }
    }

    pub fn build_script(&self) -> PathBuf {
        self.language_path.join("build.sh")
    }

    pub fn builder_config(&self) -> PathBuf {
        self.language_path.join("builder.toml")
    }

    pub fn runner_config(&self) -> PathBuf {
        self.language_path.join("runner.toml")
    }
}
