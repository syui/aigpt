//src/main.rs
mod model;
mod logic;
mod agent;
mod cli;
mod utils;
mod commands;

use cli::cli_app;
use seahorse::App;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let app: App = cli_app();
    app.run(args);
}
