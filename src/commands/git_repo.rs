// src/commands/git_repo.rs
use std::fs;

// Gitリポジトリ内の全てのファイルを取得し、内容を読み取る
pub fn read_all_git_files(repo_path: &str) -> String {
    let mut content = String::new();
    for entry in fs::read_dir(repo_path).expect("ディレクトリ読み込み失敗") {
        let entry = entry.expect("エントリ読み込み失敗");
        let path = entry.path();
        if path.is_file() {
            if let Ok(file_content) = fs::read_to_string(&path) {
                content.push_str(&format!("\n\n# File: {}\n{}", path.display(), file_content));
            }
        }
    }
    content
}
