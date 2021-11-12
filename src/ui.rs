use crate::user_env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::time::{Duration, Instant};

use egui_backend::sdl2::video::GLProfile;
use egui_backend::{egui, sdl2};
use egui_backend::{sdl2::event::Event, sdl2::event::EventType, DpiScaling};
use egui_sdl2_gl as egui_backend;
use sdl2::video::{GLContext, SwapInterval};

extern crate image;
use image::GenericImageView;

use crate::run_context::RunContext;
use crate::run_context::SteamControllerEvent;
use crate::run_context::ThreadCommand;

const PROMPT_CONTROLLER_Y: &[u8] = include_bytes!("../res/prompts/Steam_Y.png");
const PROMPT_CONTROLLER_A: &[u8] = include_bytes!("../res/prompts/Steam_A.png");
const PROMPT_CONTROLLER_X: &[u8] = include_bytes!("../res/prompts/Steam_X.png");
const PROMPT_CONTROLLER_B: &[u8] = include_bytes!("../res/prompts/Steam_B.png");

const PROMPT_CONTROLLER_DUALSHOCK_Y: &[u8] = include_bytes!("../res/prompts/PS4_Y.png");
const PROMPT_CONTROLLER_DUALSHOCK_A: &[u8] = include_bytes!("../res/prompts/PS4_A.png");
const PROMPT_CONTROLLER_DUALSHOCK_X: &[u8] = include_bytes!("../res/prompts/PS4_X.png");
const PROMPT_CONTROLLER_DUALSHOCK_B: &[u8] = include_bytes!("../res/prompts/PS4_B.png");

const PROMPT_KEYBOARD_SPACE: &[u8] = include_bytes!("../res/prompts/Space_Key_Dark.png");
const PROMPT_KEYBOARD_ENTER: &[u8] = include_bytes!("../res/prompts/Enter_Key_Dark.png");
const PROMPT_KEYBOARD_ESC: &[u8] = include_bytes!("../res/prompts/Esc_Key_Dark.png");
const PROMPT_KEYBOARD_CTRL: &[u8] = include_bytes!("../res/prompts/Ctrl_Key_Dark.png");

pub const DEFAULT_WINDOW_W: u32 = 600;
pub const DEFAULT_WINDOW_H: u32 = 180;
pub const DEFAULT_PROMPT_SIZE: f32 = 32_f32;
pub const SCROLL_TIMES: usize = 40_usize;
pub const AXIS_DEAD_ZONE: i16 = 10_000;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum RequestedAction {
    Confirm,
    Back,
    CustomAction,
    SecondCustomAction,
}

#[derive(PartialEq, Copy, Clone)]
pub enum ControllerType {
    Xbox,
    DualShock,
}

pub struct EguiWindowInstance {
    window: egui_sdl2_gl::sdl2::video::Window,
    _ctx: GLContext,
    event_pump: sdl2::EventPump,
    sdl2_controller: std::option::Option<sdl2::controller::GameController>,
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
    pub last_requested_action: Option<RequestedAction>,
    pub controller_type: ControllerType,
    context: Option<std::sync::Arc<std::sync::Mutex<RunContext>>>,
    video_subsystem: egui_sdl2_gl::sdl2::VideoSubsystem,
}

impl EguiWindowInstance {
    pub fn start_egui_loop<F>(&mut self, mut egui_ctx: egui::CtxRef, mut f: F)
    where
        F: FnMut((&mut EguiWindowInstance, &egui::CtxRef)),
    {
        let mut last_axis_value = 0;
        let mut last_axis_timestamp = Instant::now();
        'running: loop {
            self.window
                .subsystem()
                .gl_set_swap_interval(SwapInterval::VSync)
                .unwrap();
            self.egui_state.input.time = Some(self.start_time.elapsed().as_secs_f64());

            let mut exit = false;
            let (egui_output, paint_cmds) =
                egui_ctx.run(self.egui_state.input.take(), |egui_ctx| {
                    let title = &self.title;

                    if self.sdl2_controller.is_none() {
                        let context_check = self.context.clone();
                        if let Some(context) = context_check {
                            let mut guard = context.lock().unwrap();
                            if let Some(event) = guard.event {
                                match event {
                                    SteamControllerEvent::Connected => {
                                        self.attached_to_controller = true;
                                    }
                                    SteamControllerEvent::Disconnected => {
                                        self.attached_to_controller = false;
                                    }
                                    SteamControllerEvent::RequestedAction(action) => {
                                        self.last_requested_action = Some(action);
                                    }
                                    SteamControllerEvent::Up => {
                                        if self.enable_nav {
                                            self.nav_counter_up += 1;
                                        }
                                    }
                                    SteamControllerEvent::Down => {
                                        if self.enable_nav {
                                            self.nav_counter_down += 1;
                                        }
                                    }
                                }
                                guard.event = None;
                            }
                            std::mem::drop(guard);
                        }
                    } else {
                        if self.enable_nav {
                            let controller = self.sdl2_controller.as_ref().unwrap();
                            let axis_value = controller.axis(sdl2::controller::Axis::LeftY);
                            if axis_value == last_axis_value {
                                last_axis_timestamp = Instant::now();
                            }
                            else if last_axis_timestamp.elapsed().as_millis() >= 300 {
                                if axis_value > AXIS_DEAD_ZONE || axis_value < -AXIS_DEAD_ZONE {
                                    last_axis_timestamp = Instant::now();
                                    last_axis_value = axis_value;

                                    if axis_value < 0 {
                                        self.nav_counter_up += 1;
                                    } else {
                                        self.nav_counter_down += 1;
                                    }
                                }
                            }
                        }
                    }

                    if let Some(last_requested_action) = self.last_requested_action {
                        if last_requested_action == RequestedAction::Back {
                            exit = true;
                            self.from_exit = true;
                            self.last_requested_action = None;
                        }
                    }

                    egui::TopBottomPanel::top("title_bar")
                        .frame(default_panel_frame())
                        .resizable(false)
                        .show(egui_ctx, |ui| {
                            let layout = egui::Layout::bottom_up(egui::Align::Center)
                                .with_cross_justify(true);
                            ui.with_layout(layout, |ui| {
                                ui.separator();
                                ui.vertical_centered(|ui| {
                                    ui.label(title);
                                });
                            });
                        });

                    f((self, egui_ctx));
                });

            if exit {
                break;
            }

            self.egui_state.process_output(&egui_output);

            let paint_jobs = egui_ctx.tessellate(paint_cmds);

            if !egui_output.needs_repaint {
                std::thread::sleep(Duration::from_millis(10))
            }

            self.painter
                .paint_jobs(None, paint_jobs, &egui_ctx.texture());
            self.window.gl_swap_window();

            if !egui_output.needs_repaint {
                if let Some(event) = self.event_pump.wait_event_timeout(5) {
                    match event {
                        Event::Quit { .. } => break 'running,
                        Event::ControllerButtonUp { button, .. } => {
                            if button == sdl2::controller::Button::DPadUp {
                                if self.enable_nav {
                                    self.nav_counter_up += 1;
                                }
                            } else if button == sdl2::controller::Button::DPadDown {
                                if self.enable_nav {
                                    self.nav_counter_down += 1;
                                }
                            } else if button == sdl2::controller::Button::A {
                                self.last_requested_action = Some(RequestedAction::Confirm);
                            } else if button == sdl2::controller::Button::B {
                                self.last_requested_action = Some(RequestedAction::Back);
                            } else if button == sdl2::controller::Button::Y {
                                self.last_requested_action = Some(RequestedAction::CustomAction);
                            } else if button == sdl2::controller::Button::X {
                                self.last_requested_action =
                                    Some(RequestedAction::SecondCustomAction);
                            }
                        }
                        Event::KeyDown { keycode, .. } => {
                            if let Some(keycode) = keycode {
                                match keycode {
                                    sdl2::keyboard::Keycode::Down => {
                                        if self.enable_nav {
                                            self.nav_counter_down += 1;
                                        }
                                    }
                                    sdl2::keyboard::Keycode::Up => {
                                        if self.enable_nav {
                                            self.nav_counter_up += 1;
                                        }
                                    }
                                    sdl2::keyboard::Keycode::S => {
                                        if self.enable_nav {
                                            self.nav_counter_down += 1;
                                        }
                                    }
                                    sdl2::keyboard::Keycode::W => {
                                        if self.enable_nav {
                                            self.nav_counter_up += 1;
                                        }
                                    }
                                    sdl2::keyboard::Keycode::Return => {
                                        self.last_requested_action = Some(RequestedAction::Confirm);
                                    }
                                    sdl2::keyboard::Keycode::Escape => {
                                        self.last_requested_action = Some(RequestedAction::Back);
                                    }
                                    sdl2::keyboard::Keycode::Space => {
                                        self.last_requested_action =
                                            Some(RequestedAction::CustomAction);
                                    }
                                    sdl2::keyboard::Keycode::LCtrl => {
                                        self.last_requested_action =
                                            Some(RequestedAction::SecondCustomAction);
                                    }
                                    _ => {
                                        self.egui_state.process_input(
                                            &self.window,
                                            event,
                                            &mut self.painter,
                                        );
                                    }
                                };
                            }
                        }
                        Event::ControllerDeviceRemoved { .. } => {
                            self.attached_to_controller = false;
                        }
                        _ => {
                            self.egui_state
                                .process_input(&self.window, event, &mut self.painter);
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
                                    self.nav_counter_up += 1;
                                }
                            } else if button == sdl2::controller::Button::DPadDown {
                                if self.enable_nav {
                                    self.nav_counter_down += 1;
                                }
                            } else if button == sdl2::controller::Button::A {
                                self.last_requested_action = Some(RequestedAction::Confirm);
                            } else if button == sdl2::controller::Button::B {
                                self.last_requested_action = Some(RequestedAction::Back);
                            } else if button == sdl2::controller::Button::Y {
                                self.last_requested_action = Some(RequestedAction::CustomAction);
                            } else if button == sdl2::controller::Button::X {
                                self.last_requested_action =
                                    Some(RequestedAction::SecondCustomAction);
                            }
                        }
                        Event::KeyDown { keycode, .. } => {
                            if let Some(keycode) = keycode {
                                match keycode {
                                    sdl2::keyboard::Keycode::Down => {
                                        if self.enable_nav {
                                            self.nav_counter_down += 1;
                                        }
                                    }
                                    sdl2::keyboard::Keycode::Up => {
                                        if self.enable_nav {
                                            self.nav_counter_up += 1;
                                        }
                                    }
                                    sdl2::keyboard::Keycode::S => {
                                        if self.enable_nav {
                                            self.nav_counter_down += 1;
                                        }
                                    }
                                    sdl2::keyboard::Keycode::W => {
                                        if self.enable_nav {
                                            self.nav_counter_up += 1;
                                        }
                                    }
                                    sdl2::keyboard::Keycode::Return => {
                                        self.last_requested_action = Some(RequestedAction::Confirm);
                                    }
                                    sdl2::keyboard::Keycode::Escape => {
                                        self.last_requested_action = Some(RequestedAction::Back);
                                    }
                                    sdl2::keyboard::Keycode::Space => {
                                        self.last_requested_action =
                                            Some(RequestedAction::CustomAction);
                                    }
                                    sdl2::keyboard::Keycode::LCtrl => {
                                        self.last_requested_action =
                                            Some(RequestedAction::SecondCustomAction);
                                    }
                                    _ => {
                                        self.egui_state.process_input(
                                            &self.window,
                                            event,
                                            &mut self.painter,
                                        );
                                    }
                                };
                            }
                        }
                        Event::ControllerDeviceRemoved { .. } => {
                            self.attached_to_controller = false;
                        }
                        _ => {
                            self.egui_state
                                .process_input(&self.window, event, &mut self.painter);
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

    pub fn get_clipboard_contents(&mut self) -> Result<String, Error> {
        match self.video_subsystem.clipboard().clipboard_text() {
            Ok(clipboard_text) => Ok(clipboard_text),
            Err(_err) => Err(Error::new(ErrorKind::Other, "clipboard contents error")),
        }
    }
}

pub fn start_egui_window(
    window_width: u32,
    window_height: u32,
    window_title: &str,
    enable_nav: bool,
    context: Option<std::sync::Arc<std::sync::Mutex<RunContext>>>,
) -> Result<(EguiWindowInstance, egui::CtxRef), Error> {
    sdl2::hint::set("SDL_HINT_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR", "0");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 2);

    let mut window_flags: u32 = 0;
    window_flags |= sdl2::sys::SDL_WindowFlags::SDL_WINDOW_UTILITY as u32;
    window_flags |= sdl2::sys::SDL_WindowFlags::SDL_WINDOW_ALWAYS_ON_TOP as u32;
    window_flags |= sdl2::sys::SDL_WindowFlags::SDL_WINDOW_RESIZABLE as u32;

    let mut window = video_subsystem
        .window(window_title, window_width, window_height)
        .set_window_flags(window_flags)
        .opengl()
        .borderless()
        .build()
        .unwrap();

    window.raise();

    let _ctx = window.gl_create_context().unwrap();
    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 2));

    let egui_ctx = egui::CtxRef::default();

    let mut event_pump = sdl_context.event_pump().unwrap();
    event_pump.disable_event(EventType::JoyAxisMotion);
    event_pump.disable_event(EventType::ControllerAxisMotion);

    let mut attached_to_controller = false;
    let mut try_steam_controller = false;
    let mut controller_type = ControllerType::Xbox;
    let game_controller_subsystem = sdl_context.game_controller().unwrap();
    let mut sdl2_controller = None; //needed for controller connection to stay alive

    let config_json_file = user_env::tool_dir().join("config.json");
    let config_json_str = fs::read_to_string(config_json_file)?;
    let config_parsed = json::parse(&config_json_str).unwrap();

    let mut use_controller = true;
    let mut use_steam_controller = true;
    if !config_parsed["use_controller"].is_null() {
        use_controller = config_parsed["use_controller"] == true;
    }
    if !config_parsed["use_steam_controller"].is_null() {
        use_steam_controller = config_parsed["use_steam_controller"] == true;
    }

    if use_controller {
        match game_controller_subsystem.num_joysticks() {
            Ok(available) => {
                println!("{} joysticks available", available);

                if available == 0 {
                    let controller_context = context.clone();
                    if let Some(controller_context) = controller_context {
                        let guard = controller_context.lock().unwrap();
                        if let Some(SteamControllerEvent::Connected) = guard.last_connected_event {
                            attached_to_controller = true;
                        }
                        std::mem::drop(guard);
                    }
                }

                if let Some(found_controller) = (0..available).find_map(|id| {
                    if !game_controller_subsystem.is_game_controller(id) {
                        println!("{} is not a game controller", id);
                        return None;
                    }

                    println!("Attempting to open controller {}", id);

                    match game_controller_subsystem.name_for_index(id) {
                        Ok(name) => {
                            println!("controller name is {}", name);
                            if name == "Steam Virtual Gamepad" {
                                try_steam_controller = true;
                            }
                        }
                        Err(err) => {
                            println!("controller name request failed: {:?}", err);
                        }
                    };

                    if try_steam_controller {
                        return None;
                    }

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
                    println!(
                        "Controller connected mapping: {}",
                        found_controller.mapping()
                    );

                    if found_controller.name().contains("PS3")
                        || found_controller.name().contains("PS4")
                        || found_controller.name().contains("PS5")
                    {
                        println!("controller assumed to be dualshock");
                        controller_type = ControllerType::DualShock;
                    } else {
                        println!("controller assumed to be xbox");
                    }

                    sdl2_controller = Some(found_controller);
                    attached_to_controller = true;
                }
            }
            Err(err) => {
                println!("num_joysticks error {}", err);
            }
        }
    } else {
        println!("controller support disabled");
    }

    let controller_context = context.clone();
    if let Some(controller_context) = controller_context {
        let mut guard = controller_context.lock().unwrap();
        if try_steam_controller && !attached_to_controller && use_steam_controller {
            guard.thread_command = Some(ThreadCommand::Connect);
        } else if sdl2_controller.is_some() || !use_controller || !use_steam_controller {
            guard.thread_command = Some(ThreadCommand::Stop);
            if !use_steam_controller {
                println!("steam controller support disabled");
            }
        }
        std::mem::drop(guard);
    }

    let (painter, egui_state) = egui_backend::with_sdl2(&window, DpiScaling::Custom(1.1));
    let start_time = Instant::now();
    Ok((
        EguiWindowInstance {
            window,
            _ctx,
            event_pump,
            sdl2_controller,
            painter,
            egui_state,
            start_time,
            should_close: false,
            title: window_title.to_string(),
            from_exit: false,
            enable_nav,
            nav_counter_down: 0,
            nav_counter_up: 0,
            attached_to_controller,
            last_requested_action: None,
            controller_type,
            context,
            video_subsystem,
        },
        egui_ctx,
    ))
}

pub fn egui_with_prompts(
    yes_button: bool,
    no_button: bool,
    yes_text: &str,
    no_text: &str,
    title: &str,
    message: &str,
    mut window_height: u32,
    button_text: &str,
    button_message: bool,
    timeout_in_seconds: i8,
    context: Option<std::sync::Arc<std::sync::Mutex<RunContext>>>,
) -> Result<(bool, bool), Error> {
    if window_height == 0 {
        window_height = DEFAULT_WINDOW_H;
    }
    let (mut window, egui_ctx) =
        start_egui_window(DEFAULT_WINDOW_W, window_height, title, true, context)?;
    let mut no = false;
    let mut yes = false;
    let mut last_attached_state = window.attached_to_controller;

    let mut last_current_scroll = 0 as f32;
    let mut last_max_scroll = 0 as f32;

    let mut texture_confirm = prompt_image_for_action(RequestedAction::Confirm, &mut window)
        .unwrap()
        .0;
    let mut texture_back = prompt_image_for_action(RequestedAction::Back, &mut window)
        .unwrap()
        .0;
    let prompt_vec = egui::vec2(DEFAULT_PROMPT_SIZE, DEFAULT_PROMPT_SIZE);

    window.start_egui_loop(egui_ctx, |(window_instance, egui_ctx)| {
        if let Some(last_requested_action) = window_instance.last_requested_action {
            if last_requested_action == RequestedAction::Confirm {
                yes = true;
            }
            window_instance.last_requested_action = None;
        }

        if (!window_instance.attached_to_controller && last_attached_state)
            || (window_instance.attached_to_controller && !last_attached_state)
        {
            println!("Detected controller change, reloading prompts");
            texture_confirm = prompt_image_for_action(RequestedAction::Confirm, window_instance)
                .unwrap()
                .0;
            texture_back = prompt_image_for_action(RequestedAction::Back, window_instance)
                .unwrap()
                .0;
            last_attached_state = window_instance.attached_to_controller;
        }

        let mut requested_scroll_up = 0;
        let mut requested_scroll_down = 0;
        if window_instance.enable_nav
            && (window_instance.nav_counter_down != 0 || window_instance.nav_counter_up != 0)
        {
            if window_instance.nav_counter_down != 0 {
                requested_scroll_down = window_instance.nav_counter_down;
                window_instance.nav_counter_down = 0;
            } else {
                requested_scroll_up = window_instance.nav_counter_up;
                window_instance.nav_counter_up = 0;
            }
        }

        egui::TopBottomPanel::bottom("bottom_panel")
            .frame(default_panel_frame())
            .resizable(false)
            .show(egui_ctx, |ui| {
                let layout = egui::Layout::top_down(egui::Align::Center).with_cross_justify(true);
                ui.with_layout(layout, |ui| {
                    if button_message {
                        ui.label(&button_text.to_string());
                    }
                    ui.separator();
                });

                egui::SidePanel::right("Right Panel")
                    .frame(egui::Frame::none())
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        let layout = egui::Layout::right_to_left().with_cross_justify(true);
                        ui.with_layout(layout, |ui| {
                            if no_button
                                && ui
                                    .add(egui::Button::image_and_text(
                                        texture_back,
                                        prompt_vec,
                                        no_text,
                                    ))
                                    .clicked()
                            {
                                no = true;
                            }

                            if yes_button
                                && ui
                                    .add(egui::Button::image_and_text(
                                        texture_confirm,
                                        prompt_vec,
                                        yes_text,
                                    ))
                                    .clicked()
                            {
                                yes = true;
                            }
                        });
                    });
            });

        let mut seconds_left = 0 as f64;
        if timeout_in_seconds != 0 {
            let seconds = window_instance.start_time.elapsed().as_secs_f64();
            seconds_left = timeout_in_seconds as f64 - seconds;
        }

        egui::CentralPanel::default().show(egui_ctx, |ui| {
            let mut scroll_area = egui::ScrollArea::vertical();
            if requested_scroll_down != 0 {
                let calculated_scroll =
                    last_current_scroll + (requested_scroll_down * SCROLL_TIMES) as f32;
                if calculated_scroll <= last_max_scroll {
                    scroll_area = scroll_area.scroll_offset(egui::vec2(0.0, calculated_scroll));
                }
            } else if requested_scroll_up != 0 {
                let calculated_scroll =
                    last_current_scroll - (requested_scroll_up * SCROLL_TIMES) as f32;
                if calculated_scroll >= 0.0 {
                    scroll_area = scroll_area.scroll_offset(egui::vec2(0.0, calculated_scroll));
                }
            }

            let (current_scroll, max_scroll) = scroll_area.show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    if timeout_in_seconds == 0 {
                        ui.label(&message.to_string());
                    } else {
                        ui.label(std::format!(
                            "Launching\n{}\nin {:.0} seconds",
                            message,
                            seconds_left
                        ));
                    }
                });

                let margin = ui.visuals().clip_rect_margin;
                let current_scroll = ui.clip_rect().top() - ui.min_rect().top() + margin;
                let max_scroll = ui.min_rect().height() - ui.clip_rect().height() + 2.0 * margin;
                (current_scroll, max_scroll)
            });

            last_current_scroll = current_scroll;
            last_max_scroll = max_scroll;
        });

        if timeout_in_seconds != 0 && seconds_left <= 0.0 {
            window_instance.close();
        }

        if yes || no {
            window_instance.close();
        }
    });

    if window.from_exit {
        no = true;
    }

    Ok((yes, no))
}

pub fn default_panel_frame() -> egui::Frame {
    egui::Frame {
        margin: egui::Vec2::new(8.0, 2.0),
        corner_radius: 0.0,
        fill: egui::Color32::from_gray(27),
        stroke: egui::Stroke::new(0.0, egui::Color32::from_gray(60)),
        shadow: egui::epaint::Shadow {
            extrusion: 0.0,
            color: egui::Color32::from_gray(27),
        },
    }
}

fn image_as_texture(
    image_data: &[u8],
    window_instance: &mut EguiWindowInstance,
) -> (egui::TextureId, usize, usize) {
    let image = image::load_from_memory(image_data).expect("Failed to load image");
    let image_buffer = image.to_rgba8();

    let pixels = image_buffer.into_vec();
    let pixels: Vec<_> = pixels
        .chunks_exact(4)
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    let texture_id = window_instance.painter.new_user_texture(
        (image.width() as usize, image.height() as usize),
        &pixels,
        false,
    );

    (texture_id, image.width() as usize, image.height() as usize)
}

pub fn prompt_image_for_action(
    action: RequestedAction,
    window_instance: &mut EguiWindowInstance,
) -> Result<(egui::TextureId, usize, usize), Error> {
    let image;
    match action {
        RequestedAction::Confirm => {
            if window_instance.attached_to_controller {
                if window_instance.controller_type == ControllerType::DualShock {
                    image = PROMPT_CONTROLLER_DUALSHOCK_A;
                } else {
                    image = PROMPT_CONTROLLER_A;
                }
            } else {
                image = PROMPT_KEYBOARD_ENTER;
            }
        }
        RequestedAction::Back => {
            if window_instance.attached_to_controller {
                if window_instance.controller_type == ControllerType::DualShock {
                    image = PROMPT_CONTROLLER_DUALSHOCK_B;
                } else {
                    image = PROMPT_CONTROLLER_B;
                }
            } else {
                image = PROMPT_KEYBOARD_ESC;
            }
        }
        RequestedAction::CustomAction => {
            if window_instance.attached_to_controller {
                if window_instance.controller_type == ControllerType::DualShock {
                    image = PROMPT_CONTROLLER_DUALSHOCK_Y;
                } else {
                    image = PROMPT_CONTROLLER_Y;
                }
            } else {
                image = PROMPT_KEYBOARD_SPACE;
            }
        }
        RequestedAction::SecondCustomAction => {
            if window_instance.attached_to_controller {
                if window_instance.controller_type == ControllerType::DualShock {
                    image = PROMPT_CONTROLLER_DUALSHOCK_X;
                } else {
                    image = PROMPT_CONTROLLER_X;
                }
            } else {
                image = PROMPT_KEYBOARD_CTRL;
            }
        }
    };

    Ok(image_as_texture(image, window_instance))
}
