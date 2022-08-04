use crate::user_env;
use log::{debug, error, info};
use std::env;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::io::{Error, ErrorKind};
use std::time::Instant;

use arboard::Clipboard;
use egui_extras::RetainedImage;
use egui_glow::glow;
extern crate sdl2;
use glutin::platform::run_return::EventLoopExtRunReturn;
use sdl2::{event::Event, event::EventType};

extern crate image;
use crate::LUX_STEAM_DECK;
use crate::LUX_STEAM_DECK_GAMING_MODE;

const PROMPT_CONTROLLER_Y: &[u8] = include_bytes!("../res/prompts/Steam_Y.png");
const PROMPT_CONTROLLER_A: &[u8] = include_bytes!("../res/prompts/Steam_A.png");
const PROMPT_CONTROLLER_X: &[u8] = include_bytes!("../res/prompts/Steam_X.png");
const PROMPT_CONTROLLER_B: &[u8] = include_bytes!("../res/prompts/Steam_B.png");

const PROMPT_CONTROLLER_DUALSHOCK_Y: &[u8] = include_bytes!("../res/prompts/PS4_Y.png");
const PROMPT_CONTROLLER_DUALSHOCK_A: &[u8] = include_bytes!("../res/prompts/PS4_A.png");
const PROMPT_CONTROLLER_DUALSHOCK_X: &[u8] = include_bytes!("../res/prompts/PS4_X.png");
const PROMPT_CONTROLLER_DUALSHOCK_B: &[u8] = include_bytes!("../res/prompts/PS4_B.png");

const PROMPT_CONTROLLER_SWITCH_Y: &[u8] = include_bytes!("../res/prompts/Switch_Y.png");
const PROMPT_CONTROLLER_SWITCH_A: &[u8] = include_bytes!("../res/prompts/Switch_A.png");
const PROMPT_CONTROLLER_SWITCH_X: &[u8] = include_bytes!("../res/prompts/Switch_X.png");
const PROMPT_CONTROLLER_SWITCH_B: &[u8] = include_bytes!("../res/prompts/Switch_B.png");

const PROMPT_KEYBOARD_SPACE: &[u8] = include_bytes!("../res/prompts/Space_Key_Dark.png");
const PROMPT_KEYBOARD_ENTER: &[u8] = include_bytes!("../res/prompts/Enter_Key_Dark.png");
const PROMPT_KEYBOARD_ESC: &[u8] = include_bytes!("../res/prompts/Esc_Key_Dark.png");
const PROMPT_KEYBOARD_CTRL: &[u8] = include_bytes!("../res/prompts/Ctrl_Key_Dark.png");

const PROMPT_CONTROLLER_STEAM_DECK_Y: &[u8] = include_bytes!("../res/prompts/SteamDeck_Y.png");
const PROMPT_CONTROLLER_STEAM_DECK_A: &[u8] = include_bytes!("../res/prompts/SteamDeck_A.png");
const PROMPT_CONTROLLER_STEAM_DECK_X: &[u8] = include_bytes!("../res/prompts/SteamDeck_X.png");
const PROMPT_CONTROLLER_STEAM_DECK_B: &[u8] = include_bytes!("../res/prompts/SteamDeck_B.png");

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

impl Display for ControllerType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ControllerType::Xbox => {
                write!(f, "Xbox")
            }
            ControllerType::DualShock => {
                write!(f, "DualShock")
            }
            ControllerType::Switch => {
                write!(f, "Switch")
            }
            ControllerType::SteamDeck => {
                write!(f, "SteamDeck")
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum ControllerType {
    Xbox,
    DualShock,
    Switch,
    SteamDeck,
}

pub struct EguiWindowInstance {
    egui_glow: egui_glow::EguiGlow,
    gl_window: glutin::WindowedContext<glutin::PossiblyCurrent>,
    gl: std::rc::Rc<glow::Context>,
    event_loop: glutin::event_loop::EventLoop<()>,
    event_pump: sdl2::EventPump,
    sdl2_controller: std::option::Option<sdl2::controller::GameController>,
    pub window_data: EguiWindowInstanceData,
}

pub struct EguiWindowInstanceData {
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
    pub reload_requested: bool,
}

impl EguiWindowInstance {
    pub fn start_egui_loop<F>(&mut self, _egui_ctx: egui::Context, mut f: F)
    where
        F: FnMut((&mut EguiWindowInstanceData, &egui::Context)),
    {
        let mut last_axis_value = 0;
        let mut last_axis_timestamp = Instant::now();
        let mut last_input_timestamp = Instant::now();

        let clear_color = [0.1, 0.1, 0.1];

        let Self {
            gl_window,
            gl,
            egui_glow,
            window_data,
            sdl2_controller,
            event_pump,
            ..
        } = self;

        let title = &window_data.title.to_owned();

        self.event_loop.run_return(move |event, _, control_flow| {
            let mut exit = false;

            if sdl2_controller.is_some() && last_input_timestamp.elapsed().as_millis() >= 300 {
                let controller = sdl2_controller.as_ref().unwrap();
                let axis_value = controller.axis(sdl2::controller::Axis::LeftY);
                if axis_value == last_axis_value {
                    last_axis_timestamp = Instant::now();
                } else if last_axis_timestamp.elapsed().as_millis() >= 300
                    && last_input_timestamp.elapsed().as_millis() >= 300
                    && (axis_value > AXIS_DEAD_ZONE || axis_value < -AXIS_DEAD_ZONE)
                {
                    last_axis_timestamp = Instant::now();
                    last_input_timestamp = Instant::now();
                    last_axis_value = axis_value;

                    if axis_value < 0 {
                        window_data.nav_counter_up += 1;
                        window_data.reload_requested = true;
                    } else {
                        window_data.nav_counter_down += 1;
                        window_data.reload_requested = true;
                    }
                }

                if let Some(last_requested_action) = window_data.last_requested_action {
                    if last_requested_action == RequestedAction::Back {
                        exit = true;
                        window_data.from_exit = true;
                        window_data.last_requested_action = None;
                    }
                }
            }

            let mut redraw = || {
                let needs_repaint = egui_glow.run(gl_window.window(), |egui_ctx| {
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

                    f((window_data, egui_ctx));
                });

                if needs_repaint {
                    gl_window.window().request_redraw();
                    glutin::event_loop::ControlFlow::Poll
                } else {
                    glutin::event_loop::ControlFlow::Wait
                };

                {
                    unsafe {
                        use glow::HasContext as _;
                        gl.clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
                        gl.clear(glow::COLOR_BUFFER_BIT);
                    }

                    egui_glow.paint(gl_window.window());
                    gl_window.swap_buffers().unwrap();
                }
            };

            match event {
                glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
                glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::KeyboardInput {
                        input:
                            glutin::event::KeyboardInput {
                                virtual_keycode: Some(virtual_code),
                                state,
                                ..
                            },
                        ..
                    } => {
                        if last_input_timestamp.elapsed().as_millis() >= 300 {
                            last_input_timestamp = Instant::now();
                            match (virtual_code, state) {
                                (
                                    glutin::event::VirtualKeyCode::Down,
                                    glutin::event::ElementState::Pressed,
                                ) => {
                                    if window_data.enable_nav {
                                        window_data.nav_counter_down += 1;
                                        window_data.reload_requested = true;
                                    }
                                }
                                (
                                    glutin::event::VirtualKeyCode::Up,
                                    glutin::event::ElementState::Pressed,
                                ) => {
                                    if window_data.enable_nav {
                                        window_data.nav_counter_up += 1;
                                        window_data.reload_requested = true;
                                    }
                                }
                                (
                                    glutin::event::VirtualKeyCode::S,
                                    glutin::event::ElementState::Pressed,
                                ) => {
                                    if window_data.enable_nav {
                                        window_data.nav_counter_down += 1;
                                        window_data.reload_requested = true;
                                    }
                                }
                                (
                                    glutin::event::VirtualKeyCode::W,
                                    glutin::event::ElementState::Pressed,
                                ) => {
                                    if window_data.enable_nav {
                                        window_data.nav_counter_up += 1;
                                        window_data.reload_requested = true;
                                    }
                                }
                                (
                                    glutin::event::VirtualKeyCode::Return,
                                    glutin::event::ElementState::Pressed,
                                ) => {
                                    window_data.last_requested_action =
                                        Some(RequestedAction::Confirm);
                                    window_data.reload_requested = true;
                                }
                                (
                                    glutin::event::VirtualKeyCode::Escape,
                                    glutin::event::ElementState::Pressed,
                                ) => {
                                    window_data.last_requested_action = Some(RequestedAction::Back);
                                    window_data.reload_requested = true;
                                }
                                (
                                    glutin::event::VirtualKeyCode::Space,
                                    glutin::event::ElementState::Pressed,
                                ) => {
                                    window_data.last_requested_action =
                                        Some(RequestedAction::CustomAction);
                                    window_data.reload_requested = true;
                                }
                                (
                                    glutin::event::VirtualKeyCode::LControl,
                                    glutin::event::ElementState::Pressed,
                                ) => {
                                    window_data.last_requested_action =
                                        Some(RequestedAction::SecondCustomAction);
                                    window_data.reload_requested = true;
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => {
                        use glutin::event::WindowEvent;
                        if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                            *control_flow = glutin::event_loop::ControlFlow::Exit;
                        }

                        if let glutin::event::WindowEvent::Resized(physical_size) = event {
                            gl_window.resize(physical_size);
                        }

                        egui_glow.on_event(&event);
                        gl_window.window().request_redraw();
                    }
                },
                glutin::event::Event::LoopDestroyed => {
                    egui_glow.destroy();
                }
                _ => (),
            }

            if sdl2_controller.is_some() {
                if let Some(event) = event_pump.poll_event() {
                    match event {
                        Event::ControllerButtonUp { button, .. } => {
                            if last_input_timestamp.elapsed().as_millis() >= 300 {
                                last_input_timestamp = Instant::now();
                                if button == sdl2::controller::Button::DPadUp {
                                    if window_data.enable_nav {
                                        window_data.nav_counter_up += 1;
                                        window_data.reload_requested = true;
                                    }
                                } else if button == sdl2::controller::Button::DPadDown {
                                    if window_data.enable_nav {
                                        window_data.nav_counter_down += 1;
                                        window_data.reload_requested = true;
                                    }
                                } else if button == sdl2::controller::Button::A {
                                    window_data.last_requested_action =
                                        Some(RequestedAction::Confirm);
                                    window_data.reload_requested = true;
                                } else if button == sdl2::controller::Button::B {
                                    window_data.last_requested_action = Some(RequestedAction::Back);
                                    window_data.reload_requested = true;
                                } else if button == sdl2::controller::Button::Y {
                                    window_data.last_requested_action =
                                        Some(RequestedAction::CustomAction);
                                    window_data.reload_requested = true;
                                } else if button == sdl2::controller::Button::X {
                                    window_data.last_requested_action =
                                        Some(RequestedAction::SecondCustomAction);
                                    window_data.reload_requested = true;
                                }
                            }
                        }
                        Event::ControllerDeviceRemoved { .. } => {
                            window_data.attached_to_controller = false;
                            window_data.reload_requested = true;
                        }
                        _ => {}
                    }
                }
            }

            if exit || window_data.should_close {
                *control_flow = glutin::event_loop::ControlFlow::Exit;
                egui_glow.destroy();
                drop(gl_window.window());
            }

            if window_data.reload_requested {
                window_data.reload_requested = false;
                gl_window.window().request_redraw();
            }
        });
    }
}

impl EguiWindowInstanceData {
    pub fn close(&mut self) {
        self.should_close = true;
    }

    pub fn get_clipboard_contents(&mut self) -> Result<String, Error> {
        let mut clipboard = Clipboard::new().unwrap();
        match clipboard.get_text() {
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
) -> Result<(EguiWindowInstance, egui::Context), Error> {
    let egui_ctx = egui::Context::default();

    let mut on_steam_deck = false;
    let mut steam_deck_gaming_mode = true;
    match env::var(LUX_STEAM_DECK) {
        Ok(val) => {
            if val == "1" {
                on_steam_deck = true;
            }
        }
        Err(err) => {
            debug!("LUX_STEAM_DECK env not found: {}", err);
        }
    }

    match env::var(LUX_STEAM_DECK_GAMING_MODE) {
        Ok(val) => {
            if val != "1" {
                steam_deck_gaming_mode = false;
            }
        }
        Err(err) => {
            debug!("LUX_STEAM_DECK_GAMING_MODE env not found: {}", err);
            steam_deck_gaming_mode = false;
        }
    }

    info!(
        "on_steam_deck: {} steam_deck_gaming_mode: {}",
        on_steam_deck, steam_deck_gaming_mode
    );

    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    event_pump.disable_event(EventType::JoyAxisMotion);
    event_pump.disable_event(EventType::ControllerAxisMotion);

    let mut attached_to_controller = false;
    let mut controller_type = ControllerType::Xbox;
    let game_controller_subsystem = sdl_context.game_controller().unwrap();
    let mut sdl2_controller = None; //needed for controller connection to stay alive

    let config_json_file = user_env::tool_dir().join("config.json");
    let config_json_str = fs::read_to_string(config_json_file)?;
    let config_parsed = json::parse(&config_json_str).unwrap();

    let mut use_controller = true;
    if !config_parsed["use_controller"].is_null() {
        use_controller = config_parsed["use_controller"] == true;
    }

    if use_controller {
        match game_controller_subsystem.num_joysticks() {
            Ok(available) => {
                info!("{} joysticks available", available);

                if let Some(found_controller) = (0..available).find_map(|id| {
                    if !game_controller_subsystem.is_game_controller(id) {
                        error!("{} is not a game controller", id);
                        return None;
                    }

                    info!("Attempting to open controller {}", id);

                    let mut ignore_controller = false;

                    match game_controller_subsystem.name_for_index(id) {
                        Ok(name) => {
                            info!("controller name is {}", name);
                            if name == "Steam Virtual Gamepad"
                                && (!on_steam_deck || !steam_deck_gaming_mode)
                            {
                                info!("ignorning steam virtual gamepad");
                                ignore_controller = true;
                            }
                        }
                        Err(err) => {
                            error!("controller name request failed: {:?}", err);
                        }
                    };

                    if ignore_controller {
                        return None;
                    }

                    match game_controller_subsystem.open(id) {
                        Ok(c) => {
                            info!("Success: opened \"{}\"", c.name());
                            Some(c)
                        }
                        Err(e) => {
                            error!("failed: {:?}", e);
                            None
                        }
                    }
                }) {
                    info!(
                        "Controller connected mapping: {}",
                        found_controller.mapping()
                    );

                    if found_controller.name().contains("PS3")
                        || found_controller.name().contains("PS4")
                        || found_controller.name().contains("PS5")
                    {
                        info!("controller assumed to be dualshock");
                        controller_type = ControllerType::DualShock;
                    } else if found_controller.name().contains("Pro") {
                        info!("controller assumed to be switch");
                        controller_type = ControllerType::Switch;
                    } else if found_controller.name() == "Steam Virtual Gamepad" && on_steam_deck {
                        info!("controller assumed to be steam deck");
                        controller_type = ControllerType::SteamDeck;
                    } else {
                        info!("controller assumed to be xbox");
                    }

                    sdl2_controller = Some(found_controller);
                    attached_to_controller = true;
                }
            }
            Err(err) => {
                error!("num_joysticks error {}", err);
            }
        }
    } else {
        error!("controller support disabled");
    }

    if attached_to_controller {
        user_env::set_controller_var(&controller_type.to_string());
    } else {
        user_env::set_controller_var("");
    }

    let event_loop: glutin::event_loop::EventLoop<()> =
        glutin::event_loop::EventLoop::with_user_event();
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(false)
        .with_inner_size(glutin::dpi::PhysicalSize {
            width: window_width,
            height: window_height,
        })
        .with_decorations(false)
        .with_always_on_top(true)
        .with_title(window_title);

    let gl_window = unsafe {
        glutin::ContextBuilder::new()
            .with_depth_buffer(0)
            .with_srgb(true)
            .with_stencil_buffer(0)
            .with_vsync(true)
            .build_windowed(window_builder, &event_loop)
            .unwrap()
            .make_current()
            .unwrap()
    };

    let gl = unsafe {
        glow::Context::from_loader_function(|s| gl_window.get_proc_address(s) as *const _)
    };
    let gl = std::rc::Rc::new(gl);

    unsafe {
        use glow::HasContext as _;
        gl.enable(glow::FRAMEBUFFER_SRGB);
    }

    let egui_glow = egui_glow::EguiGlow::new(gl_window.window(), gl.clone());
    egui_glow.egui_ctx.set_pixels_per_point(1.2);

    let start_time = Instant::now();
    let window_data = EguiWindowInstanceData {
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
        reload_requested: false,
    };
    Ok((
        EguiWindowInstance {
            event_loop,
            event_pump,
            sdl2_controller,
            egui_glow,
            gl,
            gl_window,
            window_data,
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
) -> Result<(bool, bool), Error> {
    if window_height == 0 {
        window_height = DEFAULT_WINDOW_H;
    }
    let (mut window, egui_ctx) = start_egui_window(DEFAULT_WINDOW_W, window_height, title, true)?;
    let mut no = false;
    let mut yes = false;
    let mut last_attached_state = window.window_data.attached_to_controller;

    let mut last_current_scroll = 0 as f32;
    let mut last_max_scroll = 0 as f32;

    let mut texture_confirm =
        prompt_image_for_action(RequestedAction::Confirm, &mut window.window_data).unwrap();
    let mut texture_back =
        prompt_image_for_action(RequestedAction::Back, &mut window.window_data).unwrap();
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
            info!("Detected controller change, reloading prompts");
            texture_confirm =
                prompt_image_for_action(RequestedAction::Confirm, window_instance).unwrap();
            texture_back = prompt_image_for_action(RequestedAction::Back, window_instance).unwrap();
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
                                        texture_back.texture_id(egui_ctx),
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
                                        texture_confirm.texture_id(egui_ctx),
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

            let mut current_scroll = 0_f32;
            let mut max_scroll = 0_f32;

            scroll_area.show(ui, |ui| {
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
                current_scroll = ui.clip_rect().top() - ui.min_rect().top() + margin;
                max_scroll = ui.min_rect().height() - ui.clip_rect().height() + 2.0 * margin;
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

    if window.window_data.from_exit {
        no = true;
    }

    Ok((yes, no))
}

pub fn default_panel_frame() -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin {
            left: 8.0,
            right: 8.0,
            top: 2.0,
            bottom: 2.0,
        },
        rounding: egui::Rounding::none(),
        outer_margin: egui::style::Margin {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        },
        fill: egui::Color32::from_gray(27),
        stroke: egui::Stroke::new(0.0, egui::Color32::from_gray(60)),
        shadow: egui::epaint::Shadow {
            extrusion: 0.0,
            color: egui::Color32::from_gray(27),
        },
    }
}

fn image_as_texture(image_data: &[u8]) -> RetainedImage {
    RetainedImage::from_image_bytes("image.png", image_data).unwrap()
}

pub fn prompt_image_for_action(
    action: RequestedAction,
    window_instance: &mut EguiWindowInstanceData,
) -> Result<RetainedImage, Error> {
    let image;
    match action {
        RequestedAction::Confirm => {
            if window_instance.attached_to_controller {
                if window_instance.controller_type == ControllerType::DualShock {
                    image = PROMPT_CONTROLLER_DUALSHOCK_A;
                } else if window_instance.controller_type == ControllerType::Switch {
                    image = PROMPT_CONTROLLER_SWITCH_A;
                } else if window_instance.controller_type == ControllerType::SteamDeck {
                    image = PROMPT_CONTROLLER_STEAM_DECK_A;
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
                } else if window_instance.controller_type == ControllerType::Switch {
                    image = PROMPT_CONTROLLER_SWITCH_B;
                } else if window_instance.controller_type == ControllerType::SteamDeck {
                    image = PROMPT_CONTROLLER_STEAM_DECK_B;
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
                } else if window_instance.controller_type == ControllerType::Switch {
                    image = PROMPT_CONTROLLER_SWITCH_Y;
                } else if window_instance.controller_type == ControllerType::SteamDeck {
                    image = PROMPT_CONTROLLER_STEAM_DECK_Y;
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
                } else if window_instance.controller_type == ControllerType::Switch {
                    image = PROMPT_CONTROLLER_SWITCH_X;
                } else if window_instance.controller_type == ControllerType::SteamDeck {
                    image = PROMPT_CONTROLLER_STEAM_DECK_X;
                } else {
                    image = PROMPT_CONTROLLER_X;
                }
            } else {
                image = PROMPT_KEYBOARD_CTRL;
            }
        }
    };

    Ok(image_as_texture(image))
}
