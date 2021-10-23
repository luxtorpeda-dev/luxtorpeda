use std::io::{Error, ErrorKind};
use std::time::{Duration, Instant};
use egui_backend::sdl2::video::GLProfile;
use egui_backend::{egui, sdl2};
use egui_backend::{sdl2::event::Event, DpiScaling};
use egui_sdl2_gl as egui_backend;
use sdl2::video::{SwapInterval,GLContext};

extern crate image;
use image::GenericImageView;

const PROMPT_CONTROLLER_Y: &'static [u8] = include_bytes!("../res/prompts/Steam_Y.png");
const PROMPT_CONTROLLER_A: &'static [u8] = include_bytes!("../res/prompts/Steam_A.png");
const PROMPT_CONTROLLER_X: &'static [u8] = include_bytes!("../res/prompts/Steam_X.png");
const PROMPT_CONTROLLER_B: &'static [u8] = include_bytes!("../res/prompts/Steam_B.png");
const PROMPT_KEYBOARD_SPACE: &'static [u8] = include_bytes!("../res/prompts/Space_Key_Dark.png");
const PROMPT_KEYBOARD_ENTER: &'static [u8] = include_bytes!("../res/prompts/Enter_Key_Dark.png");
const PROMPT_KEYBOARD_ESC: &'static [u8] = include_bytes!("../res/prompts/Esc_Key_Dark.png");
const PROMPT_KEYBOARD_CTRL: &'static [u8] = include_bytes!("../res/prompts/Ctrl_Key_Dark.png");

pub static DEFAULT_WINDOW_W: u32 = 600;
pub static DEFAULT_WINDOW_H: u32 = 180;
pub static DEFAULT_PROMPT_SIZE: f32 = 32 as f32;

#[derive(PartialEq, Copy, Clone)]
pub enum RequestedAction {
    Confirm,
    Back,
    CustomAction,
    SecondCustomAction
}

pub struct EguiWindowInstance {
    window: egui_sdl2_gl::sdl2::video::Window,
    _ctx: GLContext,
    pub egui_ctx: egui::CtxRef,
    event_pump: sdl2::EventPump,
    _controller: std::option::Option<sdl2::controller::GameController>,
    pub painter: egui_sdl2_gl::painter::Painter,
    egui_state: egui_sdl2_gl::EguiStateHandler,
    start_time: std::time::Instant,
    should_close: bool,
    pub from_exit: bool,
    title: String,
    pub enable_nav: bool,
    pub nav_counter_up: usize,
    pub nav_counter_down: usize,
    pub attached_to_controller: bool,
    pub last_requested_action: Option<RequestedAction>
}

impl EguiWindowInstance {
    pub fn start_egui_loop<F>(&mut self, mut f: F) where F: FnMut(&mut EguiWindowInstance), {
        'running: loop {
            self.window.subsystem().gl_set_swap_interval(SwapInterval::VSync).unwrap();
            self.egui_state.input.time = Some(self.start_time.elapsed().as_secs_f64());
            self.egui_ctx.begin_frame(self.egui_state.input.take());

            let title = &self.title;
            let mut exit = false;

            match self.last_requested_action {
                Some(last_requested_action) => {
                    if last_requested_action == RequestedAction::Back {
                        exit = true;
                        self.last_requested_action = None;
                    }
                }
                None => {}
            }

            if exit {
                break;
            }

            egui::TopBottomPanel::top("title_bar").frame(default_panel_frame()).resizable(false).show(&self.egui_ctx, |ui| {
                let layout = egui::Layout::bottom_up(egui::Align::Center).with_cross_justify(true);
                ui.with_layout(layout,|ui| {
                    ui.separator();
                    ui.vertical_centered(|ui| {
                        ui.label(title);
                    });
                });
            });

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
                            if button == sdl2::controller::Button::DPadUp {
                                if self.enable_nav {
                                    self.nav_counter_up = self.nav_counter_up + 1;
                                }
                            } else if button == sdl2::controller::Button::DPadDown {
                                if self.enable_nav {
                                    self.nav_counter_down = self.nav_counter_down + 1;
                                }
                            } else if button == sdl2::controller::Button::A {
                                self.last_requested_action = Some(RequestedAction::Confirm);
                            } else if button == sdl2::controller::Button::B {
                                self.last_requested_action = Some(RequestedAction::Back);
                            } else if button == sdl2::controller::Button::Y {
                                self.last_requested_action = Some(RequestedAction::CustomAction);
                            } else if button == sdl2::controller::Button::X {
                                self.last_requested_action = Some(RequestedAction::SecondCustomAction);
                            }
                        },
                        Event::KeyDown { keycode, .. } => {
                            if self.enable_nav {
                                if !keycode.is_none() {
                                    Some(match keycode.unwrap() {
                                        sdl2::keyboard::Keycode::Down => {
                                            if self.enable_nav {
                                                self.nav_counter_down = self.nav_counter_down + 1;
                                            }
                                        },
                                        sdl2::keyboard::Keycode::Up => {
                                            if self.enable_nav {
                                                self.nav_counter_up = self.nav_counter_up + 1;
                                            }
                                        },
                                        sdl2::keyboard::Keycode::Return => {
                                            self.last_requested_action = Some(RequestedAction::Confirm);
                                        },
                                        sdl2::keyboard::Keycode::Escape => {
                                            self.last_requested_action = Some(RequestedAction::Back);
                                        },
                                        sdl2::keyboard::Keycode::Space => {
                                            self.last_requested_action = Some(RequestedAction::CustomAction);
                                        },
                                        sdl2::keyboard::Keycode::LCtrl => {
                                            self.last_requested_action = Some(RequestedAction::SecondCustomAction);
                                        },
                                        _ => {}
                                    });
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
                            if button == sdl2::controller::Button::DPadUp {
                                if self.enable_nav {
                                    self.nav_counter_up = self.nav_counter_up + 1;
                                }
                            } else if button == sdl2::controller::Button::DPadDown {
                                if self.enable_nav {
                                    self.nav_counter_down = self.nav_counter_down + 1;
                                }
                            } else if button == sdl2::controller::Button::A {
                                self.last_requested_action = Some(RequestedAction::Confirm);
                            } else if button == sdl2::controller::Button::B {
                                self.last_requested_action = Some(RequestedAction::Back);
                            } else if button == sdl2::controller::Button::Y {
                                self.last_requested_action = Some(RequestedAction::CustomAction);
                            } else if button == sdl2::controller::Button::X {
                                self.last_requested_action = Some(RequestedAction::SecondCustomAction);
                            }
                        },
                        Event::KeyDown { keycode, .. } => {
                            Some(match keycode.unwrap() {
                                sdl2::keyboard::Keycode::Down => {
                                    self.nav_counter_down = self.nav_counter_down + 1;
                                },
                                sdl2::keyboard::Keycode::Up => {
                                    self.nav_counter_up = self.nav_counter_up + 1;
                                },
                                sdl2::keyboard::Keycode::Return => {
                                    self.last_requested_action = Some(RequestedAction::Confirm);
                                },
                                sdl2::keyboard::Keycode::Escape => {
                                    self.last_requested_action = Some(RequestedAction::Back);
                                },
                                sdl2::keyboard::Keycode::Space => {
                                    self.last_requested_action = Some(RequestedAction::CustomAction);
                                },
                                sdl2::keyboard::Keycode::LCtrl => {
                                    self.last_requested_action = Some(RequestedAction::SecondCustomAction);
                                },
                                _ => {}
                            });
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
}

pub fn start_egui_window(window_width: u32, window_height: u32, window_title: &str, enable_nav: bool) -> Result<EguiWindowInstance, Error> {
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

    let mut window = video_subsystem
        .window(
            &window_title,
            window_width,
            window_height,
        )
        .set_window_flags(window_flags)
        .opengl()
        .borderless()
        .build()
        .unwrap();

    window.raise();

    // Create a window context
    let _ctx = window.gl_create_context().unwrap();
    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 2));

    // Init egui stuff
    let egui_ctx = egui::CtxRef::default();
    let event_pump = sdl_context.event_pump().unwrap();

    let mut attached_to_controller = false;
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
                    attached_to_controller = true;
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
    Ok(EguiWindowInstance{window, _ctx, egui_ctx, event_pump, _controller: controller, painter, egui_state, start_time, should_close: false, title: window_title.to_string(), from_exit: false, enable_nav, nav_counter_down: 0, nav_counter_up: 0, attached_to_controller, last_requested_action: None})
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
    let mut window = start_egui_window(DEFAULT_WINDOW_W, window_height, &title, false)?;
    let mut no = false;
    let mut yes = false;

    let (texture_confirm, ..) = prompt_image_for_action(RequestedAction::Confirm, &mut window).unwrap();
    let (texture_back, ..) = prompt_image_for_action(RequestedAction::Back, &mut window).unwrap();
    let prompt_vec = egui::vec2(DEFAULT_PROMPT_SIZE, DEFAULT_PROMPT_SIZE);

    window.start_egui_loop(|window_instance| {
        match window_instance.last_requested_action {
            Some(last_requested_action) => {
                if last_requested_action == RequestedAction::Confirm {
                    yes = true;
                }
                window_instance.last_requested_action = None;
            }
            None => {}
        }

        egui::TopBottomPanel::bottom("bottom_panel").frame(default_panel_frame()).resizable(false).show(&window_instance.egui_ctx, |ui| {
            let layout = egui::Layout::top_down(egui::Align::Center).with_cross_justify(true);
            ui.with_layout(layout,|ui| {
                if button_message {
                    ui.label(&button_text.to_string());
                }
                ui.separator();
            });

            egui::SidePanel::right("Right Panel").frame(egui::Frame::none()).resizable(false).show_inside(ui, |ui| {
                let layout = egui::Layout::right_to_left().with_cross_justify(true);
                ui.with_layout(layout,|ui| {
                    if yes_button {
                        if ui.add(egui::ImageButtonWithText::new(&yes_text, texture_confirm, prompt_vec)).clicked() {
                            yes = true;
                        }
                    }

                    if no_button {
                        if ui.add(egui::ImageButtonWithText::new(&no_text, texture_back, prompt_vec)).clicked() {
                            no = true;
                        }
                    }
                });
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

pub fn default_panel_frame() -> egui::Frame {
    let frame = egui::Frame {
        margin: egui::Vec2::new(8.0, 2.0),
        corner_radius: 0.0,
        fill: egui::Color32::from_gray(24),
        stroke: egui::Stroke::new(0.0, egui::Color32::from_gray(60)),
        shadow: egui::epaint::Shadow::big_dark()
    };
    frame
}

fn image_as_texture(image_data: &[u8], window_instance: &mut EguiWindowInstance) -> (egui::TextureId, usize, usize) {
    let image = image::load_from_memory(&image_data).expect("Failed to load image");
    let image_buffer = image.to_rgba8();

    let pixels = image_buffer.into_vec();
    let pixels: Vec<_> = pixels
        .chunks_exact(4)
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    let texture_id = window_instance.painter.new_user_texture((image.width() as usize, image.height() as usize), &pixels, false);

    return (texture_id, image.width() as usize, image.height() as usize);
}

pub fn prompt_image_for_action(action: RequestedAction, window_instance: &mut EguiWindowInstance) -> Result<(egui::TextureId, usize, usize), Error> {
    let image;
    match action {
        RequestedAction::Confirm => {
            if window_instance.attached_to_controller {
                image = PROMPT_CONTROLLER_A;
            } else {
                image = PROMPT_KEYBOARD_ENTER;
            }
        },
        RequestedAction::Back => {
            if window_instance.attached_to_controller {
                image = PROMPT_CONTROLLER_B;
            } else {
                image = PROMPT_KEYBOARD_ESC;
            }
        },
        RequestedAction::CustomAction => {
            if window_instance.attached_to_controller {
                image = PROMPT_CONTROLLER_Y;
            } else {
                image = PROMPT_KEYBOARD_SPACE;
            }
        },
        RequestedAction::SecondCustomAction => {
            if window_instance.attached_to_controller {
                image = PROMPT_CONTROLLER_X;
            } else {
                image = PROMPT_KEYBOARD_CTRL;
            }
        }
        _ => {
            return Err(Error::new(ErrorKind::Other, "prompt_image_for_action, no image found."));
        }
    };

    return Ok(image_as_texture(image, window_instance));
}
