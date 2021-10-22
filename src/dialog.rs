use std::io;
use std::io::{Error, ErrorKind};
use std::fs::File;
use std::io::Read;

use crate::ui::egui_with_prompts;
use crate::ui::start_egui_window;
use crate::ui::DEFAULT_WINDOW_W;
use crate::ui::DEFAULT_WINDOW_H;

pub struct ProgressState {
    pub status: String,
    pub interval: usize,
    pub close: bool,
    pub error: bool,
    pub complete: bool,
    pub error_str: String
}

pub fn show_error(title: &String, error_message: &String) -> io::Result<()> {
    match egui_with_prompts(true, false, &"Ok".to_string(), &"".to_string(), &title, &error_message, 0, &"".to_string(), false) {
        Ok(_) => {
            Ok(())
        },
        Err(err) => {
            return Err(err);
        }
    }
}

pub fn show_choices(title: &str, column: &str, choices: &Vec<String>) -> io::Result<(String, bool)> {
    let mut window = start_egui_window(DEFAULT_WINDOW_W, 400, &title)?;
    let mut cancel = false;
    let mut ok = false;
    let mut choice = "";
    let mut default = false;

    window.start_egui_loop(|window_instance| {
        egui::CentralPanel::default().show(&window_instance.egui_ctx, |ui| {
            egui::SidePanel::left("Engine Choices").resizable(false).default_width(360.0).show_inside(ui, |ui| {
                ui.label(column);
                ui.separator();

                let layout = egui::Layout::top_down(egui::Align::Min).with_cross_justify(true);
                ui.with_layout(layout,|ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (_d_idx, d) in choices.iter().enumerate() {
                            ui.selectable_value(&mut choice, &d, &d);
                        }
                    });
                });
            });

            egui::SidePanel::right("Controls").resizable(false).default_width(160.0).show_inside(ui, |ui| {
                ui.label("");

                let layout = egui::Layout::top_down(egui::Align::Center).with_cross_justify(true);
                ui.with_layout(layout,|ui| {
                    ui.add(egui::Checkbox::new(&mut default, " Set as default?"));
                    ui.separator();

                    if ui.button("Ok").clicked() {
                        ok = true;
                    }
                    ui.separator();

                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }

                    ui.separator();
                });
            });
        });

        if cancel || ok {
            window_instance.close();
        }
    });

    if cancel {
        return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
    }

    if choice == "" {
        return Err(Error::new(ErrorKind::Other, "no choice selected"));
    }

    Ok((choice.to_string(), default))
}

pub fn show_file_with_confirm(title: &str, file_path: &str) -> io::Result<()> {
    let mut file = File::open(&file_path)?;
    let mut file_buf = vec![];
    file.read_to_end(&mut file_buf)?;
    let file_str = String::from_utf8_lossy(&file_buf);
    let file_str_milk = file_str.as_ref();

    match egui_with_prompts(true, true, &"Ok".to_string(), &"Cancel".to_string(), &title.to_string(), &file_str_milk.to_string(), 600, &"By clicking Ok below, you are agreeing to the above.".to_string(), true) {
        Ok(yes) => {
            if yes {
                Ok(())
            } else {
                return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
            }
        },
        Err(err) => {
            return Err(err);
        }
    }
}

pub fn show_question(title: &str, text: &str) -> Option<()> {
    match egui_with_prompts(true, true, &"Yes".to_string(), &"No".to_string(), &title.to_string(), &text.to_string(), 0, &"".to_string(), false) {
        Ok(yes) => {
            if yes {
                Some(())
            } else {
                return None
            }
        },
        Err(err) => {
            println!("show_question err: {:?}", err);
            return None
        }
    }
}

pub fn start_progress(arc: std::sync::Arc<std::sync::Mutex<ProgressState>>) -> Result<(), Error> {
    let guard = arc.lock().unwrap();
    let mut window = start_egui_window(DEFAULT_WINDOW_W, DEFAULT_WINDOW_H, &guard.status).unwrap();
    std::mem::drop(guard);

    window.start_egui_loop(|window_instance| {
        let mut guard = arc.lock().unwrap();

        egui::TopBottomPanel::bottom("bottom_panel").frame(window_instance.default_panel_frame()).resizable(false).show(&window_instance.egui_ctx, |ui| {
            let layout = egui::Layout::top_down(egui::Align::Center).with_cross_justify(true);
            ui.with_layout(layout,|ui| {
                ui.separator();
                if ui.button("Cancel").clicked() {
                    guard.close = true;
                }
                ui.separator();
            });
        });

        egui::CentralPanel::default().show(&window_instance.egui_ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(guard.status.to_string());
                ui.separator();

                if guard.interval == 100 {
                    guard.interval = 99;
                }

                let progress = guard.interval as f32 / 100 as f32;
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
