#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![windows_subsystem = "windows"]

use eframe;
use std::error::Error;

mod horario;

fn main() -> Result<(), Box<dyn Error>> {
    let app = horario::Horario::default();
    let options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), options)
}
