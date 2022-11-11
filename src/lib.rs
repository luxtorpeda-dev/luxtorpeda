use gdnative::prelude::*;
use godot_logger::GodotLogger;
use log::{Level, LevelFilter};

mod package;

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    GodotLogger::builder()
        .default_log_level(Level::Info)
        .init();

    handle.add_class::<package::Package>();
    handle.add_class::<package::SignalEmitter>();
}

// Macro that creates the entry-points of the dynamic library.
godot_init!(init);


