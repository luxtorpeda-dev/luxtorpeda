use godot::prelude::*;

mod client;
mod command;
mod config;
mod godot_logger;
mod package;
mod package_metadata;
mod user_env;

struct Luxtorpeda;

#[gdextension]
unsafe impl ExtensionLibrary for Luxtorpeda {}

// TODO: restore init_panic_hook
// TODO: replace signals coming in with direct function calls from godot?; rust would still send signals back to godot
