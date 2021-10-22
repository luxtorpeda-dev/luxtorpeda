use std::io::{Error};
use std::time::{Duration, Instant};
use egui_backend::sdl2::video::GLProfile;
use egui_backend::{egui, sdl2};
use egui_backend::{sdl2::event::Event, DpiScaling};
use egui_sdl2_gl as egui_backend;
use sdl2::video::{SwapInterval,GLContext};

pub static DEFAULT_WINDOW_W: u32 = 600;
pub static DEFAULT_WINDOW_H: u32 = 140;

pub struct EguiWindowInstance {
    window: egui_sdl2_gl::sdl2::video::Window,
    _ctx: GLContext,
    pub egui_ctx: egui::CtxRef,
    event_pump: sdl2::EventPump,
    controller: std::option::Option<sdl2::controller::GameController>,
    painter: egui_sdl2_gl::painter::Painter,
    egui_state: egui_sdl2_gl::EguiStateHandler,
    start_time: std::time::Instant,
    should_close: bool
}

impl EguiWindowInstance {
    pub fn start_egui_loop<F>(&mut self, mut f: F) where F: FnMut(&mut EguiWindowInstance), {
        'running: loop {
            self.window.subsystem().gl_set_swap_interval(SwapInterval::VSync).unwrap();
            self.egui_state.input.time = Some(self.start_time.elapsed().as_secs_f64());
            self.egui_ctx.begin_frame(self.egui_state.input.take());

            f(self);

            let (egui_output, paint_cmds) = self.egui_ctx.end_frame();
            self.egui_state.process_output(&egui_output);

            let paint_jobs = self.egui_ctx.tessellate(paint_cmds);

            if !egui_output.needs_repaint {
                std::thread::sleep(Duration::from_millis(10))
            }

            self.painter.paint_jobs(None, paint_jobs, &self.egui_ctx.texture());
            self.window.gl_swap_window();

            if !egui_output.needs_repaint {
                if let Some(event) = self.event_pump.wait_event_timeout(5) {
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
                                self.egui_state.process_input(&self.window, fake_event, &mut self.painter);
                            } else if button == sdl2::controller::Button::DPadUp {
                                let fake_event = sdl2::event::Event::KeyDown {
                                    keycode: Some(sdl2::keyboard::Keycode::Tab),
                                    repeat: false,
                                    timestamp: 0,
                                    window_id: 0,
                                    scancode: None,
                                    keymod: sdl2::keyboard::Mod::LSHIFTMOD
                                };
                                self.egui_state.process_input(&self.window, fake_event, &mut self.painter);
                            } else if button == sdl2::controller::Button::A {
                                let fake_event = sdl2::event::Event::KeyDown {
                                    keycode: Some(sdl2::keyboard::Keycode::Return),
                                    repeat: false,
                                    timestamp: 0,
                                    window_id: 0,
                                    scancode: None,
                                    keymod: sdl2::keyboard::Mod::NOMOD
                                };
                                self.egui_state.process_input(&self.window, fake_event, &mut self.painter);
                            }
                        },
                        Event::KeyUp {..} => {
                            match self.controller {
                                Some(_) => {},
                                None => {
                                    self.egui_state.process_input(&self.window, event, &mut self.painter)
                                }
                            }
                        },
                        Event::KeyDown {..} => {
                            match self.controller {
                                Some(_) => {},
                                None => {
                                    self.egui_state.process_input(&self.window, event, &mut self.painter)
                                }
                            }
                        },
                        _ => {
                            self.egui_state.process_input(&self.window, event, &mut self.painter);
                        }
                    }
                }
            } else {
                for event in self.event_pump.poll_iter() {
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
                                self.egui_state.process_input(&self.window, fake_event, &mut self.painter);
                            } else if button == sdl2::controller::Button::DPadUp {
                                let fake_event = sdl2::event::Event::KeyDown {
                                    keycode: Some(sdl2::keyboard::Keycode::Tab),
                                    repeat: false,
                                    timestamp: 0,
                                    window_id: 0,
                                    scancode: None,
                                    keymod: sdl2::keyboard::Mod::LSHIFTMOD
                                };
                                self.egui_state.process_input(&self.window, fake_event, &mut self.painter);
                            } else if button == sdl2::controller::Button::A {
                                let fake_event = sdl2::event::Event::KeyDown {
                                    keycode: Some(sdl2::keyboard::Keycode::Return),
                                    repeat: false,
                                    timestamp: 0,
                                    window_id: 0,
                                    scancode: None,
                                    keymod: sdl2::keyboard::Mod::NOMOD
                                };
                                self.egui_state.process_input(&self.window, fake_event, &mut self.painter);
                            }
                        },
                        Event::KeyUp {..} => {
                            match self.controller {
                                Some(_) => {},
                                None => {
                                    self.egui_state.process_input(&self.window, event, &mut self.painter)
                                }
                            }
                        },
                        Event::KeyDown {..} => {
                            match self.controller {
                                Some(_) => {},
                                None => {
                                    self.egui_state.process_input(&self.window, event, &mut self.painter)
                                }
                            }
                        },
                        _ => {
                            self.egui_state.process_input(&self.window, event, &mut self.painter);
                        }
                    }
                }
            }

            if self.should_close {
                break;
            }
        }
    }

    pub fn close(&mut self) {
        self.should_close = true;
    }

    pub fn default_panel_frame(&mut self) -> egui::Frame {
        let frame = egui::Frame {
            margin: egui::Vec2::new(8.0, 2.0),
            corner_radius: 0.0,
            fill: egui::Color32::from_gray(24),
            stroke: egui::Stroke::new(0.0, egui::Color32::from_gray(60)),
            shadow: egui::epaint::Shadow::big_dark()
        };
        frame
    }
}

pub fn start_egui_window(window_width: u32, window_height: u32, window_title: &str) -> Result<EguiWindowInstance, Error> {
    sdl2::hint::set("SDL_HINT_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR", "0");
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

    let mut window_flags: u32 = 0;
    window_flags |= sdl2::sys::SDL_WindowFlags::SDL_WINDOW_UTILITY as u32;

    let window = video_subsystem
        .window(
            &window_title,
            window_width,
            window_height,
        )
        .set_window_flags(window_flags)
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

    let (painter, egui_state) = egui_backend::with_sdl2(&window, DpiScaling::Custom(1.0));
    let start_time = Instant::now();
    Ok(EguiWindowInstance{window, _ctx, egui_ctx, event_pump, controller, painter, egui_state, start_time, should_close: false})
}

pub fn egui_with_prompts(
        yes_button: bool,
        no_button: bool,
        yes_text: &String,
        no_text: &String,
        title: &String,
        message: &String,
        mut window_height: u32,
        button_text: &String,
        button_message: bool) -> Result<bool, Error> {
    if window_height == 0 {
        window_height = DEFAULT_WINDOW_H;
    }
    let mut window = start_egui_window(DEFAULT_WINDOW_W, window_height, &title)?;
    let mut no = false;
    let mut yes = false;

    window.start_egui_loop(|window_instance| {
        egui::TopBottomPanel::bottom("bottom_panel").frame(window_instance.default_panel_frame()).resizable(false).show(&window_instance.egui_ctx, |ui| {
            let layout = egui::Layout::top_down(egui::Align::Center).with_cross_justify(true);
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

        egui::CentralPanel::default().show(&window_instance.egui_ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(&message.to_string());
                });
            });
        });

        if yes || no {
            window_instance.close();
        }
    });

    Ok(yes)
}
