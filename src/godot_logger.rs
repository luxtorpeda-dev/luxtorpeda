// Copyright 2016 Victor Brekenfeld
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Module providing the GodotLogger Implementation

use log::{LevelFilter, Log, Metadata, Record};
use simplelog::{Config, SharedLogger};

use godot::prelude::*;
use chrono::prelude::*;

/// The GodotLogger struct. Provides a very basic Logger implementation
pub struct GodotLogger {
    level: LevelFilter,
    config: Config,
}

impl GodotLogger {
    /// allows to create a new logger, that can be independently used, no matter what is globally set.
    ///
    /// no macros are provided for this case and you probably
    /// dont want to use this function, but `init()`, if you dont want to build a `CombinedLogger`.
    ///
    /// Takes the desired `Level` and `Config` as arguments. They cannot be changed later on.
    ///
    /// # Examples
    /// ```
    /// # extern crate simplelog;
    /// # use simplelog::*;
    /// # fn main() {
    /// let godot_logger = GodotLogger::new(LevelFilter::Info, Config::default());
    /// # }
    /// ```
    #[must_use]
    pub fn new(log_level: LevelFilter, config: Config) -> Box<GodotLogger> {
        Box::new(GodotLogger {
            level: log_level,
            config,
        })
    }
}

impl Log for GodotLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            let _ = log(&self.config, record);
        }
    }

    fn flush(&self) {}
}

#[inline(always)]
pub fn log(_config: &Config, record: &Record<'_>) {
    let local: DateTime<Local> = Local::now();
    let formatted_time = local.format("%Y-%m-%d %I:%M:%S %P").to_string();

    godot_print!("[{}] {} - {}", formatted_time, record.level(), record.args());
}

impl SharedLogger for GodotLogger {
    fn level(&self) -> LevelFilter {
        self.level
    }

    fn config(&self) -> Option<&Config> {
        Some(&self.config)
    }

    fn as_log(self: Box<Self>) -> Box<dyn Log> {
        Box::new(*self)
    }
}
