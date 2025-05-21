// src/git.rs
use std::process::Command;

pub fn git_status() {
    run_git_command(&["status"]);
}

pub fn git_init() {
    run_git_command(&["init"]);
}

#[allow(dead_code)]
pub fn git_commit(message: &str) {
    run_git_command(&["add", "."]);
    run_git_command(&["commit", "-m", message]);
}

#[allow(dead_code)]
pub fn git_push() {
    run_git_command(&["push"]);
}

#[allow(dead_code)]
pub fn git_pull() {
    run_git_command(&["pull"]);
}

#[allow(dead_code)]
pub fn git_branch() {
    run_git_command(&["branch"]);
}

fn run_git_command(args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .status()
        .expect("git コマンドの実行に失敗しました");

    if !status.success() {
        eprintln!("⚠️ git コマンドに失敗しました: {:?}", args);
    }
}
