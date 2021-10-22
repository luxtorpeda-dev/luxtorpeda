use std::fs;
use std::io::{Error, ErrorKind};

use crate::package::path_to_cache;
use crate::package::path_to_config;
use crate::package::get_game_info;
use crate::ui::start_egui_window;

struct MgmtItem {
    pub id: String,
    pub friendly_name: String,
    pub has_cache: bool,
    pub has_default: bool,
    pub is_game: bool,
    pub is_engine: bool
}

struct MgmtState {
    pub items: Vec<MgmtItem>,
    pub close: bool
}

fn detect_mgmt(arc: std::sync::Arc<std::sync::Mutex<MgmtState>>) -> Result<(), Error> {
    let cache_path = path_to_cache();
    let paths = fs::read_dir(cache_path).unwrap();
    let names =
        paths.filter_map(|entry| {
        entry.ok().and_then(|e|
            e.path().file_name()
            .and_then(|n| n.to_str().map(|s| String::from(s)))
        )
        }).collect::<Vec<String>>();

    let mut games: Vec<MgmtItem> = Vec::new();
    let mut engines: Vec<MgmtItem> = Vec::new();
    let mut guard = arc.lock().unwrap();

    for name in names {
        if name.contains("packages") {
            continue;
        }
        let friendly_name = name.to_string();
        let name_clone = name.to_string();
        if name.parse::<f64>().is_ok() {
            let mut new_item = MgmtItem{id: name_clone, has_cache: true, has_default: false, friendly_name: friendly_name, is_game: true, is_engine: false};
            let game_info_name = name.to_string();
            match get_game_info(&game_info_name) {
                Some(game_info) => {
                    new_item.friendly_name = game_info["game_name"].to_string().clone();
                },
                None => {
                    println!("detect_mgmt get_game_info for {}: not found", &name);
                }
            };

            if new_item.friendly_name != "Default" {
                games.push(new_item);
            }
        } else {
            engines.push(MgmtItem{id: name_clone, has_cache: true, has_default: false, friendly_name: friendly_name, is_game: false, is_engine: false});
        }
    }

    let config_path = path_to_config();
    let config_paths = fs::read_dir(config_path).unwrap();
    let config_names =
        config_paths.filter_map(|entry| {
        entry.ok().and_then(|e|
            e.path().file_name()
            .and_then(|n| n.to_str().map(|s| String::from(s)))
        )
        }).collect::<Vec<String>>();

    /*for name in config_names {
        if games.iter().any(|&i| i=="-i") {

        }
    }*/

    games.sort_by(|d1, d2| d1.friendly_name.cmp(&d2.friendly_name));
    engines.sort_by(|d1, d2| d1.friendly_name.cmp(&d2.friendly_name));

    guard.items = games;
    guard.items.append(&mut engines);

    std::mem::drop(guard);
    Ok(())
}

pub fn run_mgmt() -> Result<(), Error> {
    let mgmt_state = MgmtState{close: false, items: Vec::new()};
    let mutex = std::sync::Mutex::new(mgmt_state);
    let arc = std::sync::Arc::new(mutex);
    let detect_arc = arc.clone();

    match detect_mgmt(detect_arc) {
        Ok(()) => {},
        Err(err) => {
            println!("run_mgmt detect_mgmt error: {}", err);
            return Err(Error::new(ErrorKind::Other, "detect_mgmt failed"));
        }
    };

    let mut window = start_egui_window(1024, 768, "Luxtorpeda", false).unwrap();

    window.start_egui_loop(|window_instance| {
        let mut guard = arc.lock().unwrap();

        egui::TopBottomPanel::top("top_panel").resizable(false).show(&window_instance.egui_ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(std::format!("luxtorpeda-dev {0}", env!("CARGO_PKG_VERSION")));
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").resizable(false).show(&window_instance.egui_ctx, |ui| {
            let layout = egui::Layout::top_down(egui::Align::Center).with_cross_justify(true);
                ui.with_layout(layout,|ui| {
                if ui.button("Exit").clicked() {
                    guard.close = true;
                }
            });
        });

        egui::CentralPanel::default().show(&window_instance.egui_ctx, |ui| {
            egui::SidePanel::left("Left Panel").resizable(false).show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Left Panel");
                });
                egui::ScrollArea::vertical().id_source("root_scroll1").show(ui, |ui| {
                    egui::Grid::new("games_gri1d").striped(true).show(ui, |ui| {
                        for (i, item) in guard.items.iter().enumerate() {
                            ui.label(&item.friendly_name);
                            ui.add_enabled_ui(item.has_cache, |ui| {
                                if ui.button("Clear Cache").clicked() {
                                }
                            });
                            ui.add_enabled_ui(item.has_default, |ui| {
                                if ui.button("Clear Default").clicked() {
                                }
                            });
                            ui.end_row();
                        }
                    });
                });
            });

            egui::SidePanel::right("right_panel").resizable(false).show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Right Panel");
                });
                egui::ScrollArea::vertical().id_source("root_scroll2").show(ui, |ui| {
                    egui::Grid::new("games_gri2d").striped(true).show(ui, |ui| {
                        for (i, item) in guard.items.iter().enumerate() {
                            ui.label(&item.friendly_name);
                            ui.add_enabled_ui(item.has_cache, |ui| {
                                if ui.button("Clear Cache").clicked() {
                                }
                            });
                            ui.add_enabled_ui(item.has_default, |ui| {
                                if ui.button("Clear Default").clicked() {
                                }
                            });
                            ui.end_row();
                        }
                    });
                });
            });
        });

        if guard.close {
            window_instance.close();
        }
        std::mem::drop(guard);
    });

    Ok(())
}
