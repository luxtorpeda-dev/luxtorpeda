use std::fs;
use std::io::{Error, ErrorKind};

use crate::package::get_game_info_with_json;
use crate::package::path_to_cache;
use crate::package::path_to_config;
use crate::package::path_to_packages_file;
use crate::ui::default_panel_frame;
use crate::ui::prompt_image_for_action;
use crate::ui::start_egui_window;
use crate::ui::RequestedAction;
use crate::ui::DEFAULT_PROMPT_SIZE;

#[derive(Debug)]
struct MgmtItem {
    pub id: String,
    pub friendly_name: String,
    pub has_cache: bool,
    pub has_config: bool,
}

struct MgmtState {
    pub items: Vec<MgmtItem>,
    pub close: bool,
}

fn detect_mgmt(arc: std::sync::Arc<std::sync::Mutex<MgmtState>>) -> Result<(), Error> {
    let packages_json_file = path_to_packages_file();
    let json_str = match fs::read_to_string(packages_json_file) {
        Ok(s) => s,
        Err(err) => {
            println!("read err: {:?}", err);
            return Err(Error::new(ErrorKind::Other, "read err"));
        }
    };
    let parsed = match json::parse(&json_str) {
        Ok(j) => j,
        Err(err) => {
            println!("parsing err: {:?}", err);
            return Err(Error::new(ErrorKind::Other, "parsing err"));
        }
    };

    let cache_path = path_to_cache();
    let paths = fs::read_dir(cache_path).unwrap();
    let names = paths
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str().map(String::from))
            })
        })
        .collect::<Vec<String>>();

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
            let mut new_item = MgmtItem {
                id: name_clone,
                has_cache: true,
                has_config: false,
                friendly_name,
            };
            let game_info_name = name.to_string();
            if let Some(game_info) = get_game_info_with_json(&game_info_name, &parsed) {
                new_item.friendly_name = game_info["game_name"].to_string().clone();
                games.push(new_item);
            };
        } else {
            engines.push(MgmtItem {
                id: name_clone,
                has_cache: true,
                has_config: false,
                friendly_name,
            });
        }
    }

    let config_path = path_to_config();
    let config_paths = fs::read_dir(config_path).unwrap();
    let config_names = config_paths
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str().map(String::from))
            })
        })
        .collect::<Vec<String>>();

    for name in config_names {
        if name.parse::<f64>().is_ok() {
            match games.iter_mut().find(|i| i.id == name) {
                Some(item) => {
                    item.has_config = true;
                }
                None => {
                    let friendly_name = name.to_string();
                    let name_clone = name.to_string();
                    let mut new_item = MgmtItem {
                        id: name_clone,
                        has_cache: false,
                        has_config: true,
                        friendly_name,
                    };

                    let game_info_name = name.to_string();
                    if let Some(game_info) = get_game_info_with_json(&game_info_name, &parsed) {
                        new_item.friendly_name = game_info["game_name"].to_string().clone();
                        games.push(new_item);
                    };
                }
            }
        }
    }

    games.sort_by(|d1, d2| d1.friendly_name.cmp(&d2.friendly_name));
    engines.sort_by(|d1, d2| d1.friendly_name.cmp(&d2.friendly_name));

    guard.items = games;
    guard.items.append(&mut engines);

    std::mem::drop(guard);
    Ok(())
}

fn clear_config(mgmt_item: &mut MgmtItem) {
    println!("clear_config for: {:?}", &mgmt_item);

    let config_path = path_to_config();
    let folder_path = config_path.join(&mgmt_item.id);
    match fs::remove_dir_all(folder_path) {
        Ok(()) => {
            println!("clear_config done");
        }
        Err(err) => {
            println!("clear_config. err: {:?}", err);
        }
    }

    mgmt_item.has_config = false;
}

fn clear_cache(mgmt_item: &mut MgmtItem) {
    println!("clear_cache for: {:?}", &mgmt_item);

    let cache_path = path_to_cache();
    let folder_path = cache_path.join(&mgmt_item.id);
    match fs::remove_dir_all(folder_path) {
        Ok(()) => {
            println!("clear_cache done");
        }
        Err(err) => {
            println!("clear_cache. err: {:?}", err);
        }
    }

    mgmt_item.has_cache = false;
}

pub fn run_mgmt() -> Result<(), Error> {
    let mgmt_state = MgmtState {
        close: false,
        items: Vec::new(),
    };
    let mutex = std::sync::Mutex::new(mgmt_state);
    let arc = std::sync::Arc::new(mutex);
    let detect_arc = arc.clone();

    let mut current_choice_index = 0;
    let mut scroll_to_choice_index = 0;
    let mut reload_needed = false;

    match detect_mgmt(detect_arc) {
        Ok(()) => {}
        Err(err) => {
            println!("run_mgmt detect_mgmt error: {}", err);
            return Err(Error::new(ErrorKind::Other, "detect_mgmt failed"));
        }
    };

    let title = &std::format!("luxtorpeda-dev {0}", env!("CARGO_PKG_VERSION"));
    let (mut window, egui_ctx) = start_egui_window(1024, 768, title, true, None)?;
    let texture_back =
        prompt_image_for_action(RequestedAction::Back, &mut window.window_data).unwrap();
    let texture_confirm =
        prompt_image_for_action(RequestedAction::Confirm, &mut window.window_data).unwrap();
    let texture_custom_action =
        prompt_image_for_action(RequestedAction::CustomAction, &mut window.window_data).unwrap();
    let texture_second_custom_action =
        prompt_image_for_action(RequestedAction::SecondCustomAction, &mut window.window_data)
            .unwrap();
    let prompt_vec = egui::vec2(DEFAULT_PROMPT_SIZE, DEFAULT_PROMPT_SIZE);

    window.start_egui_loop(egui_ctx, |(window_instance, egui_ctx)| {
        if reload_needed {
            println!("run_mgmt detect_mgmt reload");
            let detect_loop_arc = arc.clone();
            match detect_mgmt(detect_loop_arc) {
                Ok(()) => {
                    current_choice_index = 0;
                }
                Err(err) => {
                    println!("run_mgmt detect_mgmt error: {}", err);
                }
            };
            reload_needed = false;
            window_instance.reload_requested = true;
        }

        let mut guard = arc.lock().unwrap();

        if window_instance.enable_nav
            && (window_instance.nav_counter_down != 0 || window_instance.nav_counter_up != 0)
        {
            if window_instance.nav_counter_down != 0 {
                current_choice_index += window_instance.nav_counter_down;
                window_instance.nav_counter_down = 0;
            } else {
                if current_choice_index == 0 {
                    current_choice_index = guard.items.len();
                } else {
                    current_choice_index -= window_instance.nav_counter_up;
                }

                if current_choice_index == 0 {
                    current_choice_index = guard.items.len();
                }

                window_instance.nav_counter_up = 0;
            }

            let current_choice_index_arr = current_choice_index - 1;
            if guard.items.len() <= current_choice_index_arr {
                current_choice_index = 1;
            }
            scroll_to_choice_index = current_choice_index;
        }

        if let Some(last_requested_action) = window_instance.last_requested_action {
            if last_requested_action == RequestedAction::CustomAction && current_choice_index != 0 {
                clear_config(&mut guard.items[current_choice_index - 1]);
            } else if last_requested_action == RequestedAction::SecondCustomAction
                && current_choice_index != 0
            {
                clear_cache(&mut guard.items[current_choice_index - 1]);
            } else if last_requested_action == RequestedAction::Confirm {
                reload_needed = true;
            }
            window_instance.last_requested_action = None;
        }

        egui::TopBottomPanel::bottom("bottom_panel")
            .frame(default_panel_frame())
            .resizable(false)
            .show(egui_ctx, |ui| {
                ui.separator();

                egui::SidePanel::left("Left Panel")
                    .frame(egui::Frame::none())
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        let layout = egui::Layout::left_to_right().with_cross_justify(true);
                        ui.with_layout(layout, |ui| {
                            ui.add_enabled_ui(
                                current_choice_index != 0
                                    && guard.items[current_choice_index - 1].has_cache,
                                |ui| {
                                    if ui
                                        .add(egui::Button::image_and_text(
                                            texture_second_custom_action.texture_id(egui_ctx),
                                            prompt_vec,
                                            "Clear Cache",
                                        ))
                                        .clicked()
                                    {
                                        clear_cache(&mut guard.items[current_choice_index - 1]);
                                    }
                                },
                            );

                            ui.add_enabled_ui(
                                current_choice_index != 0
                                    && guard.items[current_choice_index - 1].has_config,
                                |ui| {
                                    if ui
                                        .add(egui::Button::image_and_text(
                                            texture_custom_action.texture_id(egui_ctx),
                                            prompt_vec,
                                            "Clear Config",
                                        ))
                                        .clicked()
                                    {
                                        clear_config(&mut guard.items[current_choice_index - 1]);
                                    }
                                },
                            );
                        });
                    });

                egui::SidePanel::right("Right Panel")
                    .frame(egui::Frame::none())
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        let layout = egui::Layout::right_to_left().with_cross_justify(true);
                        ui.with_layout(layout, |ui| {
                            if ui
                                .add(egui::Button::image_and_text(
                                    texture_back.texture_id(egui_ctx),
                                    prompt_vec,
                                    "Exit",
                                ))
                                .clicked()
                            {
                                guard.close = true;
                            }

                            if ui
                                .add(egui::Button::image_and_text(
                                    texture_confirm.texture_id(egui_ctx),
                                    prompt_vec,
                                    "Refresh",
                                ))
                                .clicked()
                            {
                                reload_needed = true;
                            }
                        });
                    });
            });

        egui::CentralPanel::default().show(egui_ctx, |ui| {
            ui.label("Items");

            let layout = egui::Layout::top_down(egui::Align::Min).with_cross_justify(true);
            ui.with_layout(layout, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (d_idx, d) in guard.items.iter().enumerate() {
                        let label = &d.friendly_name;

                        let mut is_selected = false;
                        if current_choice_index != 0 && d_idx == current_choice_index - 1 {
                            is_selected = true;
                        }

                        let response = ui.add(egui::SelectableLabel::new(is_selected, label));
                        if scroll_to_choice_index != 0 && d_idx == current_choice_index - 1 {
                            response.scroll_to_me(Some(egui::Align::Max));
                            scroll_to_choice_index = 0;
                        }

                        if response.clicked() {
                            current_choice_index = d_idx + 1;
                        }
                    }
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
