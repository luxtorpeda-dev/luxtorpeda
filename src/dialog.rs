use std::io;
use std::io::{Error, ErrorKind};
use std::fs::File;
use std::io::Read;
use std::env;

use crate::ui::egui_with_prompts;
use crate::ui::start_egui_window;
use crate::ui::DEFAULT_WINDOW_W;
use crate::ui::DEFAULT_WINDOW_H;
use crate::ui::DEFAULT_PROMPT_SIZE;
use crate::ui::default_panel_frame;
use crate::ui::RequestedAction;
use crate::ui::prompt_image_for_action;
use crate::run_context::RunContext;

pub struct ProgressState {
    pub status: String,
    pub interval: usize,
    pub close: bool,
    pub error: bool,
    pub complete: bool,
    pub error_str: String
}

pub fn show_error(title: &str, error_message: &str, context: Option<std::sync::Arc<std::sync::Mutex<RunContext>>>) -> io::Result<()> {
    match egui_with_prompts(true, false, &"Ok".to_string(), &"".to_string(), title, error_message, 0, &"".to_string(), false, 0, context) {
        Ok((yes, no)) => {
            println!("{} {}", yes, no);
            Ok(())
        },
        Err(err) => {
            println!("{:?}", err);
            Err(err)
        }
    }
}

pub fn show_choices(title: &str, column: &str, choices: &[String], context: Option<std::sync::Arc<std::sync::Mutex<RunContext>>>) -> io::Result<(String, String)> {
    let mut window = start_egui_window(DEFAULT_WINDOW_W, 400, title, true, context)?;
    let mut cancel = false;
    let mut ok = false;
    let mut choice = "";
    let mut default_choice = "";
    let mut current_choice_index = 0;
    let mut scroll_to_choice_index = 0;
    let mut last_attached_state = window.attached_to_controller;

    let mut texture_confirm = prompt_image_for_action(RequestedAction::Confirm, &mut window).unwrap().0;
    let mut texture_back = prompt_image_for_action(RequestedAction::Back, &mut window).unwrap().0;
    let mut texture_custom_action = prompt_image_for_action(RequestedAction::CustomAction, &mut window).unwrap().0;
    let prompt_vec = egui::vec2(DEFAULT_PROMPT_SIZE, DEFAULT_PROMPT_SIZE);

    window.start_egui_loop(|window_instance| {
        if window_instance.enable_nav && (window_instance.nav_counter_down != 0 || window_instance.nav_counter_up != 0) {
            if window_instance.nav_counter_down != 0 {
                current_choice_index += window_instance.nav_counter_down;
                window_instance.nav_counter_down = 0;
            } else {
                if current_choice_index == 0 {
                    current_choice_index = choices.len();
                }
                else {
                    current_choice_index -= window_instance.nav_counter_up;
                }

                if current_choice_index == 0 {
                    current_choice_index = choices.len();
                }

                window_instance.nav_counter_up = 0;
            }

            let mut current_choice_index_arr = current_choice_index - 1;
            if choices.len() <= current_choice_index_arr {
                current_choice_index_arr = 0;
                current_choice_index = 1;
            }
            scroll_to_choice_index = current_choice_index;
            choice = &choices[current_choice_index_arr];
        }

        if let Some(last_requested_action) = window_instance.last_requested_action {
            if last_requested_action == RequestedAction::Confirm && !choice.is_empty() {
                ok = true;
            }
            else if last_requested_action == RequestedAction::CustomAction && !choice.is_empty() {
                if default_choice != choice {
                    default_choice = choice;
                } else {
                    default_choice = "";
                }
            }
            window_instance.last_requested_action = None;
        }

        if (!window_instance.attached_to_controller && last_attached_state) || (window_instance.attached_to_controller && !last_attached_state) {
            println!("Detected controller change, reloading prompts");
            texture_confirm = prompt_image_for_action(RequestedAction::Confirm, window_instance).unwrap().0;
            texture_back = prompt_image_for_action(RequestedAction::Back, window_instance).unwrap().0;
            texture_custom_action = prompt_image_for_action(RequestedAction::CustomAction, window_instance).unwrap().0;
            last_attached_state = window_instance.attached_to_controller;
        }

        egui::TopBottomPanel::bottom("bottom_panel").frame(default_panel_frame()).resizable(false).show(&window_instance.egui_ctx, |ui| {
            ui.separator();

            egui::SidePanel::left("Left Panel").frame(egui::Frame::none()).resizable(false).show_inside(ui, |ui| {
                ui.add_enabled_ui(!choice.is_empty(), |ui| {
                    let mut button_text = "Set as default";
                    if default_choice == choice && !default_choice.is_empty() {
                        button_text = "Unset as default"
                    }

                    if ui.button_with_image(texture_custom_action, prompt_vec, button_text).clicked() {
                        if default_choice != choice {
                            default_choice = choice;
                        } else {
                            default_choice = "";
                        }
                    }
                });
            });

            egui::SidePanel::right("Right Panel").frame(egui::Frame::none()).resizable(false).show_inside(ui, |ui| {
                let layout = egui::Layout::right_to_left().with_cross_justify(true);
                ui.with_layout(layout,|ui| {
                    ui.add_enabled_ui(!choice.is_empty(), |ui| {
                        if ui.button_with_image(texture_confirm, prompt_vec, "Ok").clicked() {
                            ok = true;
                        }
                    });

                    if ui.button_with_image(texture_back, prompt_vec, "Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        });

        /*egui::SidePanel::right("Right Panel 2").resizable(false).show(&window_instance.egui_ctx, |ui| {
            ui.label(choice);
        });*/

        egui::CentralPanel::default().show(&window_instance.egui_ctx, |ui| {
            ui.label(column);

            let layout = egui::Layout::top_down(egui::Align::Min).with_cross_justify(true);
            ui.with_layout(layout,|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (d_idx, d) in choices.iter().enumerate() {
                        let mut label = std::format!("{}", d);
                        if d == default_choice {
                            label = std::format!("{} (Default)", d);
                        }

                        let mut is_selected = false;
                        if current_choice_index != 0 && d_idx == current_choice_index - 1 {
                            is_selected = true;
                        }

                        let response = ui.add(egui::SelectableLabel::new(is_selected, label));
                        if scroll_to_choice_index != 0 && d_idx == current_choice_index - 1 {
                            response.scroll_to_me(egui::Align::Max);
                            scroll_to_choice_index = 0;
                        }

                        if response.clicked() {
                            current_choice_index = d_idx + 1;
                            choice = d;
                        }
                    }
                });
            });
        });

        if cancel || ok {
            window_instance.close();
        }
    });

    if !ok {
        return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
    }

    if choice.is_empty() {
        return Err(Error::new(ErrorKind::Other, "no choice selected"));
    }

    Ok((choice.to_string(), default_choice.to_string()))
}

pub fn show_file_with_confirm(title: &str, file_path: &str, context: Option<std::sync::Arc<std::sync::Mutex<RunContext>>>) -> io::Result<()> {
    let mut file = File::open(&file_path)?;
    let mut file_buf = vec![];
    file.read_to_end(&mut file_buf)?;
    let file_str = String::from_utf8_lossy(&file_buf);
    let file_str_milk = file_str.as_ref();

    match egui_with_prompts(true, true, &"Ok".to_string(), &"Cancel".to_string(), &title.to_string(), &file_str_milk.to_string(), 600, &"By clicking Ok below, you are agreeing to the above.".to_string(), true, 0, context) {
        Ok((yes, ..)) => {
            if yes {
                Ok(())
            } else {
                Err(Error::new(ErrorKind::Other, "dialog was rejected"))
            }
        },
        Err(err) => {
            Err(err)
        }
    }
}

pub fn show_question(title: &str, text: &str, context: Option<std::sync::Arc<std::sync::Mutex<RunContext>>>) -> Option<()> {
    match egui_with_prompts(true, true, &"Yes".to_string(), &"No".to_string(), &title.to_string(), &text.to_string(), 0, &"".to_string(), false, 0, context) {
        Ok((yes, ..)) => {
            if yes {
                Some(())
            } else {
                None
            }
        },
        Err(err) => {
            println!("show_question err: {:?}", err);
            None
        }
    }
}

pub fn start_progress(arc: std::sync::Arc<std::sync::Mutex<ProgressState>>, context: Option<std::sync::Arc<std::sync::Mutex<RunContext>>>) -> Result<(), Error> {
    let mut window = start_egui_window(DEFAULT_WINDOW_W, DEFAULT_WINDOW_H, "Progress", false, context).unwrap();
    let mut last_attached_state = window.attached_to_controller;

    let mut texture_back = prompt_image_for_action(RequestedAction::Back, &mut window).unwrap().0;
    let prompt_vec = egui::vec2(DEFAULT_PROMPT_SIZE, DEFAULT_PROMPT_SIZE);

    window.start_egui_loop(|window_instance| {
        if (!window_instance.attached_to_controller && last_attached_state) || (window_instance.attached_to_controller && !last_attached_state) {
            println!("Detected controller change, reloading prompts");
            texture_back = prompt_image_for_action(RequestedAction::Back, window_instance).unwrap().0;
            last_attached_state = window_instance.attached_to_controller;
        }

        let mut guard = arc.lock().unwrap();

        egui::TopBottomPanel::bottom("bottom_panel").frame(default_panel_frame()).resizable(false).show(&window_instance.egui_ctx, |ui| {
            let layout = egui::Layout::top_down(egui::Align::Center).with_cross_justify(true);
            ui.with_layout(layout,|ui| {
                ui.separator();
            });

            egui::SidePanel::right("Right Panel").frame(egui::Frame::none()).resizable(false).show_inside(ui, |ui| {
                let layout = egui::Layout::right_to_left().with_cross_justify(true);
                ui.with_layout(layout,|ui| {
                    if ui.button_with_image(texture_back, prompt_vec, "Cancel").clicked() {
                        guard.close = true;
                    }
                });
            });
        });

        egui::CentralPanel::default().show(&window_instance.egui_ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(guard.status.to_string());
                ui.separator();

                if guard.interval == 100 {
                    guard.interval = 99;
                }

                let progress = guard.interval as f32 / 100_f32;
                let progress_bar = egui::ProgressBar::new(progress)
                    .show_percentage()
                    .animate(true);
                ui.add(progress_bar);
            });
        });

        if guard.close {
            window_instance.close();
        }
        std::mem::drop(guard);
    });

    Ok(())
}

pub fn default_choice_confirmation_prompt(title: &str, text: &str, context: Option<std::sync::Arc<std::sync::Mutex<RunContext>>>) -> Option<()> {
    match egui_with_prompts(false, true, &"".to_string(), &"Clear Default".to_string(), &title.to_string(), &text.to_string(), 0, &"".to_string(), false, 4, context) {
        Ok((_yes, no)) => {
            if no {
                Some(())
            } else {
                None
            }
        },
        Err(err) => {
            println!("default_choice_confirmation_prompt err: {:?}", err);
            None
        }
    }
}

pub fn text_input(title: &str, label: &str, key: &str, context: Option<std::sync::Arc<std::sync::Mutex<RunContext>>>) -> io::Result<String> {
    let mut window = start_egui_window(DEFAULT_WINDOW_W, DEFAULT_WINDOW_H, title, false, context)?;
    let mut cancel = false;
    let mut ok = false;
    let mut text_input = String::new();
    let mut last_attached_state = window.attached_to_controller;

    let mut texture_confirm = prompt_image_for_action(RequestedAction::Confirm, &mut window).unwrap().0;
    let mut texture_back = prompt_image_for_action(RequestedAction::Back, &mut window).unwrap().0;
    let mut texture_custom_action = prompt_image_for_action(RequestedAction::CustomAction, &mut window).unwrap().0;
    let prompt_vec = egui::vec2(DEFAULT_PROMPT_SIZE, DEFAULT_PROMPT_SIZE);

    window.start_egui_loop(|window_instance| {
        if let Some(last_requested_action) = window_instance.last_requested_action {
            if last_requested_action == RequestedAction::Confirm && !text_input.is_empty() {
                ok = true;
            }
            else if last_requested_action == RequestedAction::CustomAction {
                match window_instance.get_clipboard_contents() {
                    Ok(s) => {
                        text_input = s;
                    },
                    Err(err) => {
                        println!("get_clipboard_contents error: {:?}", err);
                    }
                }
            }
            window_instance.last_requested_action = None;
        }

        if (!window_instance.attached_to_controller && last_attached_state) || (window_instance.attached_to_controller && !last_attached_state) {
            println!("Detected controller change, reloading prompts");
            texture_confirm = prompt_image_for_action(RequestedAction::Confirm, window_instance).unwrap().0;
            texture_back = prompt_image_for_action(RequestedAction::Back, window_instance).unwrap().0;
            texture_custom_action = prompt_image_for_action(RequestedAction::CustomAction, window_instance).unwrap().0;
            last_attached_state = window_instance.attached_to_controller;
        }

        let mut paste_clicked = false;

        egui::TopBottomPanel::bottom("bottom_panel").frame(default_panel_frame()).resizable(false).show(&window_instance.egui_ctx, |ui| {
            ui.separator();

            egui::SidePanel::left("Left Panel").frame(egui::Frame::none()).resizable(false).show_inside(ui, |ui| {
                if ui.button_with_image(texture_custom_action, prompt_vec, "Paste").clicked() {
                    paste_clicked = true;
                };
            });

            egui::SidePanel::right("Right Panel").frame(egui::Frame::none()).resizable(false).show_inside(ui, |ui| {
                let layout = egui::Layout::right_to_left().with_cross_justify(true);
                ui.with_layout(layout,|ui| {
                    ui.add_enabled_ui(!text_input.is_empty(), |ui| {
                        if ui.button_with_image(texture_confirm, prompt_vec, "Ok").clicked() {
                            ok = true;
                        }
                    });

                    if ui.button_with_image(texture_back, prompt_vec, "Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        });

        egui::CentralPanel::default().show(&window_instance.egui_ctx, |ui| {
            let layout = egui::Layout::top_down(egui::Align::Min).with_cross_justify(true);
            ui.with_layout(layout,|ui| {
                ui.label(label);
                ui.add(egui::TextEdit::singleline(&mut text_input));
            });
        });

        if paste_clicked {
            window_instance.last_requested_action = Some(RequestedAction::CustomAction);
        }

        if cancel || ok {
            window_instance.close();
        }
    });

    if !ok {
        return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
    }

    if !key.is_empty() {
        env::set_var(std::format!("DIALOGRESPONSE_{}", key), text_input.clone());
    }

    Ok(text_input)
}
