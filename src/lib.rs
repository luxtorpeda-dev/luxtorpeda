use gdnative::prelude::*;
use godot_logger::GodotLogger;
use log::Level;

mod user_env;
mod package;
mod command;
mod client;

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    GodotLogger::builder()
        .default_log_level(Level::Info)
        .init().unwrap();

    handle.add_class::<client::LuxClient>();
    handle.add_class::<client::SignalEmitter>();
}

// Macro that creates the entry-points of the dynamic library.
godot_init!(init);


