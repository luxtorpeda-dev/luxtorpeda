extern crate steamy_controller;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::ui::RequestedAction;
use crate::ui::AXIS_DEAD_ZONE;
use crate::user_env;

#[derive(Debug, Copy, Clone)]
pub enum ThreadCommand {
    Connect,
    Stop,
}

#[derive(Debug, Copy, Clone)]
pub enum SteamControllerEvent {
    Connected,
    Disconnected,
    RequestedAction(RequestedAction),
    Up,
    Down,
}

pub struct RunContext {
    pub event: Option<SteamControllerEvent>,
    pub thread_command: Option<ThreadCommand>,
    pub last_connected_event: Option<SteamControllerEvent>,
}

pub fn setup_run_context() -> (
    Option<std::sync::Arc<std::sync::Mutex<RunContext>>>,
    std::thread::JoinHandle<()>,
) {
    let context = RunContext {
        event: None,
        thread_command: None,
        last_connected_event: None,
    };
    let mutex = std::sync::Mutex::new(context);
    let context_arc = std::sync::Arc::new(mutex);
    let thread_arc = context_arc.clone();

    let context_thread = std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(500));

        let guard = thread_arc.lock().unwrap();
        let mut do_connect = false;
        let thread_command = guard.thread_command;
        if let Some(thread_command) = thread_command {
            match thread_command {
                ThreadCommand::Connect => {
                    do_connect = true;
                }
                ThreadCommand::Stop => {
                    std::mem::drop(guard);
                    break;
                }
            }
        };
        std::mem::drop(guard);

        if do_connect {
            match steamy_controller::Manager::new() {
                Ok(mut manager) => {
                    match manager.open() {
                        Ok(mut controller) => {
                            let mut controller_guard = thread_arc.lock().unwrap();
                            controller_guard.event = Some(SteamControllerEvent::Connected);
                            controller_guard.last_connected_event =
                                Some(SteamControllerEvent::Connected);
                            controller_guard.thread_command = None;
                            std::mem::drop(controller_guard);

                            let mut pad_time_elapsed = Instant::now();
                            let mut last_axis_value = 0;

                            let term = Arc::new(AtomicBool::new(false));
                            signal_hook::flag::register(
                                signal_hook::consts::SIGTERM,
                                Arc::clone(&term),
                            )
                            .unwrap();
                            let mut controller_already_closed = false;

                            user_env::set_controller_var("Xbox");

                            while !term.load(Ordering::Relaxed) {
                                let controller_guard = thread_arc.lock().unwrap();
                                if let Some(ThreadCommand::Stop) = controller_guard.thread_command {
                                    match controller.close() {
                                        Ok(()) => {
                                            println!("steam_controller controller closed from stop request");
                                        }
                                        Err(err) => {
                                            println!(
                                                "steamy_controller controller close error: {:?}",
                                                err
                                            );
                                        }
                                    }
                                    controller_already_closed = true;
                                    std::mem::drop(controller_guard);
                                    break;
                                };
                                std::mem::drop(controller_guard);

                                match controller.state(Duration::from_secs(0)) {
                                    Ok(state) => {
                                        if let steamy_controller::State::Input {
                                            buttons,
                                            pad,
                                            ..
                                        } = state
                                        {
                                            if pad.left.y != 0 {
                                                if pad.left.y == last_axis_value {
                                                    pad_time_elapsed = Instant::now();
                                                } else if pad_time_elapsed.elapsed().as_millis()
                                                    >= 300
                                                    && (pad.left.y > AXIS_DEAD_ZONE
                                                        || pad.left.y < -AXIS_DEAD_ZONE)
                                                {
                                                    let mut guard = thread_arc.lock().unwrap();
                                                    if pad.left.y < 0 {
                                                        guard.event =
                                                            Some(SteamControllerEvent::Down);
                                                    } else {
                                                        guard.event =
                                                            Some(SteamControllerEvent::Up);
                                                    }
                                                    last_axis_value = pad.left.y;
                                                    pad_time_elapsed = Instant::now();
                                                    std::mem::drop(guard);
                                                }
                                            }
                                            if !buttons.is_empty() {
                                                let mut guard = thread_arc.lock().unwrap();
                                                let button = buttons;
                                                {
                                                    let mut found_button = false;
                                                    if pad_time_elapsed.elapsed().as_millis() >= 300
                                                    {
                                                        if button.contains(
                                                            steamy_controller::Button::PAD_UP,
                                                        ) || button.contains(
                                                            steamy_controller::Button::PAD_DOWN,
                                                        ) {
                                                            if button.contains(
                                                                steamy_controller::Button::PAD_UP,
                                                            ) {
                                                                guard.event =
                                                                    Some(SteamControllerEvent::Up);
                                                            } else if button.contains(
                                                                steamy_controller::Button::PAD_DOWN,
                                                            ) {
                                                                guard.event = Some(
                                                                    SteamControllerEvent::Down,
                                                                );
                                                            }
                                                            found_button = true;
                                                        } else if button
                                                            .contains(steamy_controller::Button::A)
                                                        {
                                                            guard.event = Some(SteamControllerEvent::RequestedAction(RequestedAction::Confirm));
                                                            found_button = true;
                                                        } else if button
                                                            .contains(steamy_controller::Button::B)
                                                        {
                                                            guard.event = Some(SteamControllerEvent::RequestedAction(RequestedAction::Back));
                                                            found_button = true;
                                                        } else if button
                                                            .contains(steamy_controller::Button::Y)
                                                        {
                                                            guard.event = Some(SteamControllerEvent::RequestedAction(RequestedAction::CustomAction));
                                                            found_button = true;
                                                        } else if button
                                                            .contains(steamy_controller::Button::X)
                                                        {
                                                            guard.event = Some(SteamControllerEvent::RequestedAction(RequestedAction::SecondCustomAction));
                                                            found_button = true;
                                                        }

                                                        if found_button {
                                                            pad_time_elapsed = Instant::now();
                                                        }
                                                    }
                                                }
                                                std::mem::drop(guard);
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!(
                                            "steamy_controller controller state error: {:?}",
                                            err
                                        );

                                        let mut guard = thread_arc.lock().unwrap();
                                        guard.thread_command = Some(ThreadCommand::Stop);
                                        guard.event = Some(SteamControllerEvent::Disconnected);
                                        guard.last_connected_event =
                                            Some(SteamControllerEvent::Disconnected);
                                        std::mem::drop(guard);

                                        break;
                                    }
                                }
                            }

                            if !controller_already_closed {
                                match controller.close() {
                                    Ok(()) => {
                                        println!("steam_controller controller closed");
                                    }
                                    Err(err) => {
                                        println!(
                                            "steamy_controller controller close error: {:?}",
                                            err
                                        );
                                    }
                                }
                            } else {
                                println!("steam_controller controller loop ended, already closed");
                            }
                        }
                        Err(err) => {
                            println!("steamy_controller controller error: {:?}", err);
                            std::thread::sleep(Duration::from_millis(2000))
                        }
                    };
                }
                Err(err) => {
                    println!("steamy_controller manager error: {:?}", err);
                    std::thread::sleep(Duration::from_millis(2000))
                }
            };
        }
    });

    (Some(context_arc), context_thread)
}
