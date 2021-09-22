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

pub enum ProgressCreateOutput {
    Zenity(Child),
    Unused(String)
}

fn get_zenity_path() -> Result<String, Error>  {
    let zenity_path = match env::var("STEAM_ZENITY") {
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
        println!("active_dialog_command. using gtk.");
        Ok("gtk".to_string())
    }
}

pub fn show_error(title: &String, error_message: &String) -> io::Result<()> {
    if active_dialog_command(false)? == "gtk" {
        let window = Window::new(WindowType::Toplevel);
        window.connect_delete_event(|_,_| {gtk::main_quit(); Inhibit(false) });

        window.set_title(title);
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(300, 100);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
        vbox.set_homogeneous(false);
        window.add(&vbox);

        let label = gtk::Label::new(Some(error_message));
        vbox.add(&label);

        let ok_button = gtk::Button::with_label("Ok");

        let button_box = gtk::ButtonBox::new(gtk::Orientation::Horizontal);
        button_box.set_layout(gtk::ButtonBoxStyle::Center);
        button_box.pack_start(&ok_button, false, false, 0);

        let window_clone_ok = window.clone();
        ok_button.connect_clicked(move |_| {
            window_clone_ok.close();
        });

        vbox.pack_end(&button_box, false, false, 0);

        window.show_all();
        gtk::main();
    } else {
        let zenity_path = match get_zenity_path() {
            Ok(s) => s,
            Err(_) => {
                return Err(Error::new(ErrorKind::Other, "zenity path not found"))
            }
        };

        let zenity_command: Vec<String> = vec![
            "--error".to_string(),
            std::format!("--text={}", error_message).to_string(),
            std::format!("--title={}", title).to_string()
        ];

        Command::new(zenity_path)
            .args(&zenity_command)
            .env("LD_PRELOAD", "")
            .status()
            .expect("failed to show zenity error");
    }

    Ok(())
}

pub fn show_choices(title: &str, column: &str, choices: &Vec<String>) -> io::Result<String> {
    if active_dialog_command(false)? == "gtk" {
        let window = Window::new(WindowType::Toplevel);
        window.connect_delete_event(|_,_| {gtk::main_quit(); Inhibit(false) });

        window.set_title(title);
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(300, 400);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 8);
        vbox.set_homogeneous(false);
        window.add(&vbox);

        let sw = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
        sw.set_shadow_type(gtk::ShadowType::EtchedIn);
        sw.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        sw.set_vexpand(true);
        vbox.add(&sw);

        let store = TreeStore::new(&[String::static_type()]);
        let treeview = gtk::TreeView::with_model(&store);
        treeview.set_vexpand(true);

        for (_d_idx, d) in choices.iter().enumerate() {
            store.insert_with_values(None, None, &[(0, &d)]);
        }

        sw.add(&treeview);

        {
            let renderer = gtk::CellRendererText::new();
            let tree_column = gtk::TreeViewColumn::new();
            tree_column.pack_start(&renderer, true);
            tree_column.set_title(column);
            tree_column.set_sizing(gtk::TreeViewColumnSizing::Fixed);
            tree_column.add_attribute(&renderer, "text", 0);
            tree_column.set_fixed_width(50);
            treeview.append_column(&tree_column);
        }

        let window_clone_cancel = window.clone();
        let window_clone_ok = window.clone();

        let cancel_button = gtk::Button::with_label("Cancel");
        cancel_button.set_margin_end(5);
        let ok_button = gtk::Button::with_label("Ok");
        let button_box = gtk::ButtonBox::new(gtk::Orientation::Horizontal);
        button_box.set_layout(gtk::ButtonBoxStyle::End);
        button_box.pack_start(&cancel_button, false, false, 0);
        button_box.pack_start(&ok_button, false, false, 0);

        let choice: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let captured_choice = choice.clone();
        let captured_choice_cancel = choice.clone();

        treeview.selection().connect_changed(move |tree_selection| {
            let (model, iter) = tree_selection.selected().expect("Couldn't get selected");
            let choice_value = model
                    .value(&iter, 0)
                    .get::<String>()
                    .expect("Treeview selection, column 0");
            *captured_choice.borrow_mut() = Some(choice_value.to_string());
        });

        let ok_choice: Rc<RefCell<Option<()>>> = Rc::new(RefCell::new(None));
        let captured_ok_choice = ok_choice.clone();

        cancel_button.connect_clicked(move |_| {
            *captured_choice_cancel.borrow_mut() = None;
            window_clone_cancel.close();
        });

        ok_button.connect_clicked(move |_| {
            *captured_ok_choice.borrow_mut() = Some(());
            window_clone_ok.close();
        });

        vbox.pack_end(&button_box, false, false, 0);

        window.show_all();
        gtk::main();

        let ok_choice_borrow = ok_choice.borrow();
        let ok_choice_match = ok_choice_borrow.as_ref();

        match ok_choice_match {
            Some(_) => {
                let choice_borrow = choice.borrow();
                let choice_match = choice_borrow.as_ref();
                match choice_match {
                    Some(s) => {
                        let choice_str = s.to_string();
                        Ok(choice_str)
                    },
                    None => {
                        return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
                    }
                }
            },
            None => {
                return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
            }
        }
    } else {
        let mut zenity_list_command: Vec<String> = vec![
            "--list".to_string(),
            std::format!("--title={0}", title),
            std::format!("--column={0}", column),
            "--hide-header".to_string()
        ];

        for entry in choices {
            zenity_list_command.push(entry.to_string());
        }

        let zenity_path = match get_zenity_path() {
            Ok(s) => s,
            Err(_) => {
                return Err(Error::new(ErrorKind::Other, "zenity path not found"))
            }
        };

        let choice = Command::new(zenity_path)
            .args(&zenity_list_command)
            .env("LD_PRELOAD", "")
            .output()
            .expect("failed to show choices");

        if !choice.status.success() {
            return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
        }

        let choice_name = match String::from_utf8(choice.stdout) {
            Ok(s) => String::from(s.trim()),
            Err(_) => {
                return Err(Error::new(ErrorKind::Other, "Failed to parse choice name"));
            }
        };

        Ok(choice_name)
    }
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

pub fn start_progress(title: &str, status: &str, interval: usize) -> io::Result<ProgressCreateOutput> {
     let progress_command: Vec<String> = vec![
        "--progress".to_string(),
        std::format!("--title={}", title).to_string(),
        std::format!("--percentage={}",interval).to_string(),
        std::format!("--text={}", status).to_string()
    ];

    println!("progress_command {:?}", progress_command);

    let zenity_path = match get_zenity_path() {
        Ok(s) => s,
        Err(_) => {
            return Err(Error::new(ErrorKind::Other, "zenity path not found"))
        }
    };

    let progress = Command::new(zenity_path)
        .args(&progress_command)
        .env("LD_PRELOAD", "")
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    Ok(ProgressCreateOutput::Zenity(progress))
}

pub fn progress_text_change(title: &str, progress_ref: &mut ProgressCreateOutput) -> io::Result<()> {
    if let ProgressCreateOutput::Zenity(ref mut progress) = progress_ref {
        {
            let stdin = match progress.stdin.as_mut() {
                Some(s) => s,
                None => {
                    return Err(Error::new(ErrorKind::Other, "progress update label failed"));
                }
            };

            match stdin.write_all(std::format!("# {}\n", title).as_bytes()) {
                Ok(()) => {},
                Err(err) => {
                    println!("progress update label failed: {}", err);
                    return Err(Error::new(ErrorKind::Other, "progress update label failed"));
                }
            };
            drop(stdin);
        }
    }
    Ok(())
}

pub fn progress_change(value: i64, progress_ref: &mut ProgressCreateOutput) -> io::Result<()> {
    if let ProgressCreateOutput::Zenity(ref mut progress) = progress_ref {
        {
            let mut final_value = value;
            if final_value == 100 {
                final_value = 99;
            }

            let stdin = match progress.stdin.as_mut() {
                Some(s) => s,
                None => {
                    return Err(Error::new(ErrorKind::Other, "progress update failed"));
                }
            };

            match stdin.write_all(std::format!("{}\n", final_value).as_bytes()) {
                Ok(()) => {},
                Err(err) => {
                    println!("progress update failed: {}", err);
                    return Err(Error::new(ErrorKind::Other, "progress update failed"));
                }
            };
            drop(stdin);
        }
        Ok(())
    } else {
        return Err(Error::new(ErrorKind::Other, "Progress not implemented"));
    }
}

pub fn progress_close(progress_ref: &mut ProgressCreateOutput) -> io::Result<()> {
    if let ProgressCreateOutput::Zenity(ref mut progress) = progress_ref {
        progress.kill().expect("command wasn't running");
        Ok(())
    } else {
        return Err(Error::new(ErrorKind::Other, "Progress not implemented"));
    }
}
