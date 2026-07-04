#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(err) = gate_client::run() {
        eprintln!("Gate client failed: {}", err);
        std::process::exit(1);
    }
}
