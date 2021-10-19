use std::io;
use std::env;
use std::io::{Error, ErrorKind};
use std::process::Command;
use std::process::Child;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::process::Stdio;
use std::cell::RefCell;

extern crate gtk;
use std::rc::Rc;
use gtk::prelude::*;
use gtk::{Window, WindowType, TreeStore};

static STEAM_ZENITY: &str = "STEAM_ZENITY";

use std::time::{Duration, Instant};
use egui::Checkbox;
use egui_backend::sdl2::video::GLProfile;
use egui_backend::{egui, sdl2};
use egui_backend::{sdl2::event::Event, DpiScaling};
use egui_sdl2_gl as egui_backend;
use sdl2::video::{SwapInterval,GLContext};
use egui::CtxRef;

pub struct ProgressState {
    pub status: String,
    pub interval: usize,
    pub close: bool,
    pub error: bool,
    pub complete: bool,
    pub error_str: String
}

fn get_zenity_path() -> Result<String, Error>  {
    let zenity_path = match env::var(STEAM_ZENITY) {
        Ok(s) => s,
        Err(_) => {
            return Err(Error::new(ErrorKind::Other, "Path could not be found"));
        }
    };

    return Ok(zenity_path);
}

fn active_dialog_command(silent: bool) -> io::Result<String> {
    if gtk::init().is_err() {
        if !silent {
            println!("active_dialog_command. Failed to initialize GTK, using zenity.");
        }
        Ok("zenity".to_string())
    } else {
        if !silent {
            println!("active_dialog_command. using gtk.");
        }
        Ok("gtk".to_string())
    }
}

fn start_egui_window(window_width: u32, window_height: u32, window_title: &str) -> Result<(
        egui_sdl2_gl::sdl2::video::Window,
        GLContext,
        CtxRef,
        sdl2::EventPump), Error> {
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
        .resizable()
        .build()
        .unwrap();

    // Create a window context
    let _ctx = window.gl_create_context().unwrap();
    debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    debug_assert_eq!(gl_attr.context_version(), (3, 2));

    // Init egui stuff
    let egui_ctx = egui::CtxRef::default();
    let event_pump = sdl_context.event_pump().unwrap();

    Ok((window, _ctx, egui_ctx, event_pump))
}

pub fn show_error(title: &String, error_message: &String) -> io::Result<()> {
    let (window, _ctx, mut egui_ctx, mut event_pump) = start_egui_window(400, 120, &title)?;
    let (mut painter, mut egui_state) = egui_backend::with_sdl2(&window, DpiScaling::Custom(1.0));

    let mut ok = false;
    let start_time = Instant::now();

    'running: loop {
        window.subsystem().gl_set_swap_interval(SwapInterval::VSync).unwrap();

        egui_state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(egui_state.input.take());

        egui::CentralPanel::default().show(&egui_ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(&error_message.to_string());

                ui.separator();

                if ui.button("Ok").clicked() {
                    ok = true;
                }
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
                    _ => {
                        // Process input event
                        egui_state.process_input(&window, event, &mut painter);
                    }
                }
            }
        } else {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    _ => {
                        // Process input event
                        egui_state.process_input(&window, event, &mut painter);
                    }
                }
            }
        }

        if ok {
            break;
        }
    }

    Ok(())
}

pub fn show_choices(title: &str, column: &str, choices: &Vec<String>) -> io::Result<(String, bool)> {
    let (window, _ctx, mut egui_ctx, mut event_pump) = start_egui_window(300, 400, &title)?;
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
                for (_d_idx, d) in choices.iter().enumerate() {
                    ui.selectable_value(&mut choice, &d, &d);
                }
            });

            ui.separator();
            ui.add(Checkbox::new(&mut default, " Set as default?"));
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }

                if ui.button("Ok").clicked() {
                    if choice != "" {
                        ok = true;
                    }
                }
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
                    _ => {
                        // Process input event
                        egui_state.process_input(&window, event, &mut painter);
                    }
                }
            }
        } else {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    _ => {
                        // Process input event
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

    if active_dialog_command(false)? == "gtk" {
        let window = Window::new(WindowType::Toplevel);
        window.connect_delete_event(|_,_| {gtk::main_quit(); Inhibit(false) });

        window.set_title(title);
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(600, 400);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
        vbox.set_homogeneous(false);
        window.add(&vbox);

        let sw = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        sw.set_shadow_type(gtk::ShadowType::EtchedIn);
        sw.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        sw.set_vexpand(true);
        vbox.add(&sw);

        let text_view = gtk::TextView::new();
        text_view.set_wrap_mode(gtk::WrapMode::Word);
        text_view.set_cursor_visible(false);
        text_view.buffer().unwrap().set_text(&file_str);
        sw.add(&text_view);

        let label = gtk::Label::new(Some("By clicking OK below, you are agreeing to the above."));
        vbox.add(&label);

        let cancel_button = gtk::Button::with_label("Cancel");
        cancel_button.set_margin_end(5);
        let ok_button = gtk::Button::with_label("Ok");
        let button_box = gtk::ButtonBox::new(gtk::Orientation::Horizontal);
        button_box.set_layout(gtk::ButtonBoxStyle::End);
        button_box.pack_start(&cancel_button, false, false, 0);
        button_box.pack_start(&ok_button, false, false, 0);

        let window_clone_cancel = window.clone();
        let window_clone_ok = window.clone();

        let choice: Rc<RefCell<Option<()>>> = Rc::new(RefCell::new(None));
        let captured_choice_ok = choice.clone();

        cancel_button.connect_clicked(move |_| {
            window_clone_cancel.close();
        });

        ok_button.connect_clicked(move |_| {
            *captured_choice_ok.borrow_mut() = Some(());
            window_clone_ok.close();
        });

        vbox.pack_end(&button_box, false, false, 0);

        window.show_all();
        gtk::main();

        let choice_borrow = choice.borrow();
        let choice_match = choice_borrow.as_ref();

        match choice_match {
            Some(_) => {
                Ok(())
            },
            None => {
                return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
            }
        }
    } else {
        let mut converted_file = File::create("converted.txt")?;
        converted_file.write_all(file_str.as_bytes())?;

        let zenity_path = match get_zenity_path() {
            Ok(s) => s,
            Err(_) => {
                return Err(Error::new(ErrorKind::Other, "zenity path not found"))
            }
        };

        let choice = Command::new(zenity_path)
            .args(&[
                "--text-info",
                &std::format!("--title={0}", title).to_string(),
                "--filename=converted.txt"])
            .env("LD_PRELOAD", "")
            .status()
            .expect("failed to show file with confirm");

        if !choice.success() {
            return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
        }
        else {
            Ok(())
        }
    }
}

pub fn show_question(title: &str, text: &str) -> Option<()> {
    if active_dialog_command(false).ok()? == "gtk" {
        let window = Window::new(WindowType::Toplevel);
        window.connect_delete_event(|_,_| {gtk::main_quit(); Inhibit(false) });

        window.set_title(title);
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(300, 100);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
        vbox.set_homogeneous(false);
        window.add(&vbox);

        let label = gtk::Label::new(Some(text));
        vbox.add(&label);

        let cancel_button = gtk::Button::with_label("No");
        cancel_button.set_margin_end(5);
        let ok_button = gtk::Button::with_label("Yes");
        let button_box = gtk::ButtonBox::new(gtk::Orientation::Horizontal);
        button_box.set_layout(gtk::ButtonBoxStyle::End);
        button_box.pack_start(&cancel_button, false, false, 0);
        button_box.pack_start(&ok_button, false, false, 0);

        let window_clone_cancel = window.clone();
        let window_clone_ok = window.clone();

        let choice: Rc<RefCell<Option<()>>> = Rc::new(RefCell::new(None));
        let captured_choice_ok = choice.clone();

        cancel_button.connect_clicked(move |_| {
            window_clone_cancel.close();
        });

        ok_button.connect_clicked(move |_| {
            *captured_choice_ok.borrow_mut() = Some(());
            window_clone_ok.close();
        });

        vbox.pack_end(&button_box, false, false, 0);

        window.show_all();
        gtk::main();

        let choice_borrow = choice.borrow();
        let choice_match = choice_borrow.as_ref();

        match choice_match {
            Some(_) => {
                Some(())
            },
            None => {
                return None
            }
        }
    } else {
        let zenity_command: Vec<String> = vec![
            "--question".to_string(),
            std::format!("--text={}", &text),
            std::format!("--title={}", &title)
        ];

        let zenity_path = match get_zenity_path() {
            Ok(s) => s,
            Err(_) => {
                return None
            }
        };

        let question = Command::new(zenity_path)
            .args(&zenity_command)
            .env("LD_PRELOAD", "")
            .status()
            .expect("failed to show question");

        if question.success() {
            Some(())
        } else {
            return None
        }
    }
}

pub fn start_progress(arc: std::sync::Arc<std::sync::Mutex<ProgressState>>) -> Result<(), Error> {
    let guard = arc.lock().unwrap();
    let (window, _ctx, mut egui_ctx, mut event_pump) = start_egui_window(300, 100, &guard.status).unwrap();
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
                    }
                    _ => {
                        // Process input event
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
                    }
                    _ => {
                        // Process input event
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
