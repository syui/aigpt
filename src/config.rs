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

    pub fn data_file(&self, file_name: &str) -> PathBuf {
        let file_path = match file_name {
            "db" => self.base_dir.join("user.db"),
            "toml" => self.base_dir.join("user.toml"),
            "json" => self.base_dir.join("user.json"),
            _ => self.base_dir.join(format!(".{}", file_name)),
        };

        file_path
    }
   /// 設定ファイルがなければ `example.json` をコピーする
    pub fn ensure_file_exists(&self, file_name: &str, template_path: &Path) {
        let target = self.data_file(file_name);
        if !target.exists() {
            if let Err(e) = fs::copy(template_path, &target) {
                eprintln!("⚠️ 設定ファイルの初期化に失敗しました: {}", e);
            } else {
                println!("📄 {} を {} にコピーしました", template_path.display(), target.display());
            }
        }
    }
}
