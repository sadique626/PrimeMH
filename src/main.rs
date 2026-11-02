#![windows_subsystem = "windows"]

#[allow(unused)]
#[macro_use]
extern crate log;

use std::{fs::File, io::Write};
use std::path::Path;
use gui::ui::start_ui;
use localisation::localisation::Localisation;
use logger::configure_logging;
use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS}; 

mod gui;
mod livemap;
mod localisation;
mod mapgeneration;
mod memory;
mod settings;
#[path = "memory/types/mod.rs"]
mod types;
mod logger;

pub const SETTINGS_FILE: &str = "settings.toml";
pub const ITEM_FILTER_FILE: &str = "itemfilter.yml";
#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;
lazy_static! {
    pub static ref LOCALISATION: Mutex<Localisation> = Mutex::new(Localisation::new());
}

fn main() {
    
    
    configure_logging();
    unsafe { 
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
    
    log::info!("Configured logging");
    let icon = include_bytes!("./gui/images/primemh.png");
    let icon_file = Path::new("./primemh.png");
    
    let mut f = File::create(icon_file).unwrap();
    match f.write_all(icon.as_slice()) {
        Ok(_) => (),
        Err(s) => {
            if !icon_file.exists() {
                log::error!("File permission error\n{:?}", s)
            }
        },
    }
    log::info!("Added Icon");
    log::info!("Starting UI...");
    start_ui().expect("Could no initalize UI, could be an OpenGL issue");
}
