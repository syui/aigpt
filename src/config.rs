// src/config.rs
use std::fs;
use std::path::{Path, PathBuf};
use shellexpand;

pub struct ConfigPaths {
    pub base_dir: PathBuf,
}

impl ConfigPaths {
    pub fn new() -> Self {
        let app_name = env!("CARGO_PKG_NAME");
        let mut base_dir = shellexpand::tilde("~").to_string();
        base_dir.push_str(&format!("/.config/{}/", app_name));
        let base_path = Path::new(&base_dir);
        if !base_path.exists() {
            let _ = fs::create_dir_all(base_path);
        }

        ConfigPaths {
            base_dir: base_path.to_path_buf(),
        }
    }

    #[allow(dead_code)]
    pub fn data_file(&self, file_name: &str) -> PathBuf {
        let file_path = match file_name {
            "db" => self.base_dir.join("user.db"),
            "toml" => self.base_dir.join("user.toml"),
            "json" => self.base_dir.join("user.json"),
            _ => self.base_dir.join(format!(".{}", file_name)),
        };
        file_path
    }

    pub fn mcp_dir(&self) -> PathBuf {
        self.base_dir.join("mcp")
    }

    pub fn venv_path(&self) -> PathBuf {
        self.mcp_dir().join(".venv")
    }

    pub fn python_executable(&self) -> PathBuf {
        if cfg!(windows) {
            self.venv_path().join("Scripts").join("python.exe")
        } else {
            self.venv_path().join("bin").join("python")
        }
    }

    pub fn pip_executable(&self) -> PathBuf {
        if cfg!(windows) {
            self.venv_path().join("Scripts").join("pip.exe")
        } else {
            self.venv_path().join("bin").join("pip")
        }
    }
}
