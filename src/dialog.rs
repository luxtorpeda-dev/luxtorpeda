use std::io;
use std::io::{Error, ErrorKind};
use std::fs::File;
use std::io::Read;

use std::time::{Duration, Instant};
use egui_backend::sdl2::video::GLProfile;
use egui_backend::{egui, sdl2};
use egui_backend::{sdl2::event::Event, DpiScaling};
use egui_sdl2_gl as egui_backend;
use sdl2::video::{SwapInterval,GLContext};

pub struct ProgressState {
    pub status: String,
    pub interval: usize,
    pub close: bool,
    pub error: bool,
    pub complete: bool,
    pub error_str: String
}

static DEFAULT_WINDOW_W: u32 = 600;
static DEFAULT_WINDOW_H: u32 = 140;

fn start_egui_window(window_width: u32, window_height: u32, window_title: &str) -> Result<(
        egui_sdl2_gl::sdl2::video::Window,
        GLContext,
        egui::CtxRef,
        sdl2::EventPump,
        std::option::Option<sdl2::controller::GameController>), Error> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);

    // Let OpenGL know we are dealing with SRGB colors so that it
    // can do the blending correctly. Not setting the framebuffer
    // leads to darkened, oversaturated colors.
    gl_attr.set_framebuffer_srgb_compatible(true);
    gl_attr.set_double_buffer(true);
    gl_attr.set_multisample_samples(4);
    // OpenGL 3.2 is the minimum that we will support.
    gl_attr.set_context_version(3, 2);

    let window = video_subsystem
        .window(
            &window_title,
            window_width,
            window_height,
        )
        .opengl()
        .build()
        .unwrap();

    // Create a window context
    let _ctx = window.gl_create_context().unwrap();
    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 2));

    // Init egui stuff
    let egui_ctx = egui::CtxRef::default();
    let event_pump = sdl_context.event_pump().unwrap();

    let game_controller_subsystem = sdl_context.game_controller().unwrap();
    let mut controller = None; //needed for controller connection to stay alive
    match game_controller_subsystem.num_joysticks() {
        Ok(available) => {
            println!("{} joysticks available", available);

            match (0..available)
            .find_map(|id| {
                if !game_controller_subsystem.is_game_controller(id) {
                    println!("{} is not a game controller", id);
                    return None;
                }

                println!("Attempting to open controller {}", id);

                match game_controller_subsystem.open(id) {
                    Ok(c) => {
                        println!("Success: opened \"{}\"", c.name());
                        Some(c)
                    }
                    Err(e) => {
                        println!("failed: {:?}", e);
                        None
                    }
                }
            }) {
                Some(found_controller) => {
                    println!("Controller connected mapping: {}", found_controller.mapping());
                    controller = Some(found_controller);
                },
                None => {
                    println!("controller not found");
                }
            }
        },
        Err(err) => {
            println!("num_joysticks error {}", err);
        }
    }

    Ok((window, _ctx, egui_ctx, event_pump, controller))
}

fn egui_with_prompts(
        yes_button: bool,
        no_button: bool,
        yes_text: &String,
        no_text: &String,
        title: &String,
        message: &String,
        scroll_max_height: f32,
        mut window_height: u32,
        button_text: &String,
        button_message: bool) -> Result<bool, Error> {
    if window_height == 0 {
        window_height = DEFAULT_WINDOW_H;
    }
    let (window, _ctx, mut egui_ctx, mut event_pump, controller) = start_egui_window(DEFAULT_WINDOW_W, window_height, &title)?;
    let (mut painter, mut egui_state) = egui_backend::with_sdl2(&window, DpiScaling::Custom(1.0));

    let mut no = false;
    let mut yes = false;
    let start_time = Instant::now();

    'running: loop {
        window.subsystem().gl_set_swap_interval(SwapInterval::VSync).unwrap();

        egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(egui_state.input.take());

        egui::CentralPanel::default().show(&egui_ctx, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false; 2]).max_height(scroll_max_height).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(&message.to_string());
                });
            });

            egui::TopBottomPanel::bottom("bottom_panel")
                .resizable(false)
                .frame(egui::Frame::none())
                .min_height(0.0)
                .show_inside(ui, |ui| {
                    let layout = egui::Layout::top_down(egui::Align::Center)
                    .with_cross_justify(true);
                    ui.with_layout(layout,|ui| {
                        if button_message {
                            ui.label(&button_text.to_string());
                        }

                        if yes_button {
                            ui.separator();
                            if ui.button(&yes_text).clicked() {
                                yes = true;
                            }
                        }

                        if no_button {
                            ui.separator();
                            if ui.button(&no_text).clicked() {
                                no = true;
                            }
                        }

                        ui.separator();
                    });
                });
        });

        let (egui_output, paint_cmds) = egui_ctx.end_frame();
        egui_state.process_output(&egui_output);

        let paint_jobs = egui_ctx.tessellate(paint_cmds);

        if !egui_output.needs_repaint {
            std::thread::sleep(Duration::from_millis(10))
        }

        painter.paint_jobs(None, paint_jobs, &egui_ctx.texture());

        window.gl_swap_window();

        if !egui_output.needs_repaint {
            if let Some(event) = event_pump.wait_event_timeout(5) {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::ControllerButtonUp { button, .. } => {
                        if button == sdl2::controller::Button::DPadDown {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Tab),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::NOMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        } else if button == sdl2::controller::Button::DPadUp {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Tab),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::LSHIFTMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        } else if button == sdl2::controller::Button::A {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Return),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::NOMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        }
                    },
                    Event::KeyUp {..} => {
                        match controller {
                            Some(_) => {},
                            None => {
                                egui_state.process_input(&window, event, &mut painter)
                            }
                        }
                    },
                    Event::KeyDown {..} => {
                        match controller {
                            Some(_) => {},
                            None => {
                                egui_state.process_input(&window, event, &mut painter)
                            }
                        }
                    },
                    _ => {
                        egui_state.process_input(&window, event, &mut painter);
                    }
                }
            }
        } else {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::ControllerButtonUp { button, .. } => {
                        if button == sdl2::controller::Button::DPadDown {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Tab),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::NOMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        } else if button == sdl2::controller::Button::DPadUp {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Tab),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::LSHIFTMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        } else if button == sdl2::controller::Button::A {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Return),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::NOMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        }
                    },
                    Event::KeyUp {..} => {
                        match controller {
                            Some(_) => {},
                            None => {
                                egui_state.process_input(&window, event, &mut painter)
                            }
                        }
                    },
                    Event::KeyDown {..} => {
                        match controller {
                            Some(_) => {},
                            None => {
                                egui_state.process_input(&window, event, &mut painter)
                            }
                        }
                    },
                    _ => {
                        egui_state.process_input(&window, event, &mut painter);
                    }
                }
            }
        }

        if no || yes {
            break;
        }
    }

    Ok(yes)
}

pub fn show_error(title: &String, error_message: &String) -> io::Result<()> {
    match egui_with_prompts(true, false, &"Ok".to_string(), &"".to_string(), &title, &error_message, 30.0, 0, &"".to_string(), false) {
        Ok(_) => {
            Ok(())
        },
        Err(err) => {
            return Err(err);
        }
    }
}

pub fn show_choices(title: &str, column: &str, choices: &Vec<String>) -> io::Result<(String, bool)> {
    let (window, _ctx, mut egui_ctx, mut event_pump, controller) = start_egui_window(300, 400, &title)?;
    let (mut painter, mut egui_state) = egui_backend::with_sdl2(&window, DpiScaling::Custom(1.0));

    let mut cancel = false;
    let mut ok = false;
    let mut choice = "";
    let mut default = false;

    let start_time = Instant::now();

    'running: loop {
        window.subsystem().gl_set_swap_interval(SwapInterval::VSync).unwrap();

        egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(egui_state.input.take());

        egui::CentralPanel::default().show(&egui_ctx, |ui| {
            ui.vertical(|ui| {
                ui.label(column);
                ui.separator();
            });

            egui::ScrollArea::vertical().auto_shrink([false; 2]).max_height(160.0).show(ui, |ui| {
                for (_d_idx, d) in choices.iter().enumerate() {
                    ui.selectable_value(&mut choice, &d, &d);
                }
            });

            ui.separator();
            ui.add(egui::Checkbox::new(&mut default, " Set as default?"));
            ui.separator();

            egui::TopBottomPanel::bottom("bottom_panel")
                .resizable(false)
                .frame(egui::Frame::none())
                .min_height(0.0)
                .show_inside(ui, |ui| {
                    let layout = egui::Layout::top_down(egui::Align::Center)
                    .with_cross_justify(true);
                    ui.with_layout(layout,|ui| {
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

        let (egui_output, paint_cmds) = egui_ctx.end_frame();
        egui_state.process_output(&egui_output);

        let paint_jobs = egui_ctx.tessellate(paint_cmds);

        if !egui_output.needs_repaint {
            std::thread::sleep(Duration::from_millis(10))
        }

        painter.paint_jobs(None, paint_jobs, &egui_ctx.texture());

        window.gl_swap_window();

        if !egui_output.needs_repaint {
            if let Some(event) = event_pump.wait_event_timeout(5) {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::ControllerButtonUp { button, .. } => {
                        if button == sdl2::controller::Button::DPadDown {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Tab),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::NOMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        } else if button == sdl2::controller::Button::DPadUp {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Tab),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::LSHIFTMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        } else if button == sdl2::controller::Button::A {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Return),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::NOMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        }
                    },
                    Event::KeyUp {..} => {
                        match controller {
                            Some(_) => {},
                            None => {
                                egui_state.process_input(&window, event, &mut painter)
                            }
                        }
                    },
                    Event::KeyDown {..} => {
                        match controller {
                            Some(_) => {},
                            None => {
                                egui_state.process_input(&window, event, &mut painter)
                            }
                        }
                    },
                    _ => {
                        egui_state.process_input(&window, event, &mut painter)
                    }
                }
            }
        } else {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::ControllerButtonUp { button, .. } => {
                        if button == sdl2::controller::Button::DPadDown {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Tab),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::NOMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        } else if button == sdl2::controller::Button::DPadUp {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Tab),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::LSHIFTMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        } else if button == sdl2::controller::Button::A {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Return),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::NOMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        }
                    },
                    Event::KeyUp {..} => {
                        match controller {
                            Some(_) => {},
                            None => {
                                egui_state.process_input(&window, event, &mut painter)
                            }
                        }
                    },
                    Event::KeyDown {..} => {
                        match controller {
                            Some(_) => {},
                            None => {
                                egui_state.process_input(&window, event, &mut painter)
                            }
                        }
                    },
                    _ => {
                        egui_state.process_input(&window, event, &mut painter);
                    }
                }
            }
        }

        if cancel || ok {
            break;
        }
    }

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

    match egui_with_prompts(true, true, &"Ok".to_string(), &"Cancel".to_string(), &title.to_string(), &file_str_milk.to_string(), 380.0, 600, &"By clicking Ok below, you are agreeing to the above.".to_string(), true) {
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
    match egui_with_prompts(true, true, &"Yes".to_string(), &"No".to_string(), &title.to_string(), &text.to_string(), 30.0, 0, &"".to_string(), false) {
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
    let (window, _ctx, mut egui_ctx, mut event_pump, controller) = start_egui_window(DEFAULT_WINDOW_W, DEFAULT_WINDOW_H, &guard.status).unwrap();
    let (mut painter, mut egui_state) = egui_backend::with_sdl2(&window, DpiScaling::Custom(1.0));
    std::mem::drop(guard);

    let start_time = Instant::now();

    'running: loop {
        window.subsystem().gl_set_swap_interval(SwapInterval::VSync).unwrap();
        let mut guard = arc.lock().unwrap();

        egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(egui_state.input.take());

        egui::CentralPanel::default().show(&egui_ctx, |ui| {
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

            egui::TopBottomPanel::bottom("bottom_panel")
                .resizable(false)
                .frame(egui::Frame::none())
                .min_height(0.0)
                .show_inside(ui, |ui| {
                    let layout = egui::Layout::top_down(egui::Align::Center)
                    .with_cross_justify(true);
                    ui.with_layout(layout,|ui| {
                        ui.separator();
                        if ui.button("Cancel").clicked() {
                            guard.close = true;
                        }
                        ui.separator();
                    });
                });
        });

        let (egui_output, paint_cmds) = egui_ctx.end_frame();
        egui_state.process_output(&egui_output);

        let paint_jobs = egui_ctx.tessellate(paint_cmds);

        if !egui_output.needs_repaint {
            std::thread::sleep(Duration::from_millis(10))
        }

        painter.paint_jobs(None, paint_jobs, &egui_ctx.texture());

        window.gl_swap_window();

        if !egui_output.needs_repaint {
            if let Some(event) = event_pump.wait_event_timeout(5) {
                match event {
                    Event::Quit { .. } => {
                        std::mem::drop(guard);
                        break 'running;
                    },
                    Event::ControllerButtonUp { button, .. } => {
                        if button == sdl2::controller::Button::DPadDown {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Tab),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::NOMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        } else if button == sdl2::controller::Button::DPadUp {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Tab),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::LSHIFTMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        } else if button == sdl2::controller::Button::A {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Return),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::NOMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        }
                    },
                    Event::KeyUp {..} => {
                        match controller {
                            Some(_) => {},
                            None => {
                                egui_state.process_input(&window, event, &mut painter)
                            }
                        }
                    },
                    Event::KeyDown {..} => {
                        match controller {
                            Some(_) => {},
                            None => {
                                egui_state.process_input(&window, event, &mut painter)
                            }
                        }
                    },
                    _ => {
                        egui_state.process_input(&window, event, &mut painter);
                    }
                }
            }
        } else {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => {
                        std::mem::drop(guard);
                        break 'running;
                    },
                    Event::ControllerButtonUp { button, .. } => {
                        if button == sdl2::controller::Button::DPadDown {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Tab),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::NOMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        } else if button == sdl2::controller::Button::DPadUp {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Tab),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::LSHIFTMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        } else if button == sdl2::controller::Button::A {
                            let fake_event = sdl2::event::Event::KeyDown {
                                keycode: Some(sdl2::keyboard::Keycode::Return),
                                repeat: false,
                                timestamp: 0,
                                window_id: 0,
                                scancode: None,
                                keymod: sdl2::keyboard::Mod::NOMOD
                            };
                            egui_state.process_input(&window, fake_event, &mut painter);
                        }
                    },
                    Event::KeyUp {..} => {
                        match controller {
                            Some(_) => {},
                            None => {
                                egui_state.process_input(&window, event, &mut painter)
                            }
                        }
                    },
                    Event::KeyDown {..} => {
                        match controller {
                            Some(_) => {},
                            None => {
                                egui_state.process_input(&window, event, &mut painter)
                            }
                        }
                    },
                    _ => {
                        egui_state.process_input(&window, event, &mut painter);
                    }
                }
            }
        }

        if guard.close {
            std::mem::drop(guard);
            break
        }
        std::mem::drop(guard);
    }

    Ok(())
}
