use godot::prelude::*;

mod client;
mod command;
mod config;
mod godot_logger;
mod package;
mod package_metadata;
mod user_env;
mod parsers;
mod proton_handler;
struct Luxtorpeda;

#[gdextension]
unsafe impl ExtensionLibrary for Luxtorpeda {}
