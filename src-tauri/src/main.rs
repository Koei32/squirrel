// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Ok, Result};

fn main() -> Result<()> {
    squirrel_lib::run();
    Ok(())
}
