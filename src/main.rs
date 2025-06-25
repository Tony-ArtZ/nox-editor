mod fm;
mod renderer;
mod utils;

use std::{
    env,
    io::{self, Read},
};

use nox_editor::{Buffer, InputAction, InputResult};

use crate::{fm::open_file, utils::*};

fn handle_save_as(file_manager: &mut nox_editor::FileManager, path: &str) {
    if path.is_empty() {
        return;
    }

    match file_manager.save_as(path) {
        Ok(_) => {
            // Update file info to the new path
            file_manager.file_info.path = path.to_string();
            file_manager.file_info.name = path.split('/').last().unwrap_or("unknown").to_string();

            file_manager.add_toast(
                &format!("File saved as: {}", path),
                3000,
                nox_editor::ToastType::Success,
            );
        }
        Err(e) => {
            file_manager.add_toast(
                &format!("Error saving file: {}", e),
                5000,
                nox_editor::ToastType::Error,
            );
        }
    }
}

fn main() {
    let mut stdin = io::stdin();

    set_terminal_raw_mode().expect("Failed to set terminal to raw mode");
    clear_screen();

    let args: Vec<String> = env::args().collect();

    let mut data: Vec<String>;
    let name: String;
    let path: String;

    if args.len() < 2 {
        data = vec![String::new()];
        name = "Untitled".to_string();
        path = "/".to_string();
    } else {
        data = open_file(&args[1]).expect("Failed to open file");
        name = args[1].split('/').last().unwrap_or("unknown").to_string();
        path = args[1].to_string();
    }

    if data.len() == 0 {
        data = vec![String::new()];
    }

    let buf = Buffer::new(data);

    let file_info = nox_editor::FileInfo { name, path };

    let mut file_manager = nox_editor::FileManager::new(buf, file_info);

    file_manager.add_toast("Welcome to Nox Editor!", 5000, nox_editor::ToastType::Info);

    renderer::render(&file_manager);

    let mut buffer = [0; 1];
    loop {
        // Update toasts before handling input (remove expired toasts)
        file_manager.update_toasts();

        if file_manager.input_handler.taking_input {
            if stdin.read(&mut buffer).is_ok() {
                // Handle input mode keys
                match file_manager.input_handler.handle_key(buffer[0]) {
                    InputResult::Confirmed(input) => {
                        // Process the confirmed input based on action type
                        match file_manager.input_handler.action_type {
                            InputAction::SaveAs => handle_save_as(&mut file_manager, &input),
                            InputAction::Generic => {
                                file_manager.add_toast(
                                    &format!("Received input: {}", input),
                                    3000,
                                    nox_editor::ToastType::Info,
                                );
                            }
                        }
                    }
                    InputResult::Cancelled => {
                        file_manager.add_toast(
                            "Operation cancelled",
                            2000,
                            nox_editor::ToastType::Info,
                        );
                    }
                    InputResult::InProgress => {}
                }
                renderer::render(&file_manager);
            }
            continue;
        }

        if stdin.read(&mut buffer).is_ok() {
            match buffer[0] {
                // Handle Arrow Keys
                b'\x1b' => {
                    let mut arrow_buffer = [0; 2];
                    if stdin.read(&mut arrow_buffer).is_ok() {
                        match arrow_buffer {
                            [b'[', b'A'] => file_manager.move_pointer(-1, 0), // Up arrow - decrease y
                            [b'[', b'B'] => file_manager.move_pointer(1, 0), // Down arrow - increase y
                            [b'[', b'C'] => file_manager.move_pointer(0, 1), // Right arrow - increase x
                            [b'[', b'D'] => file_manager.move_pointer(0, -1), // Left arrow - decrease x

                            // Alt+S for Save As
                            [b's', _] => {
                                file_manager
                                    .input_handler
                                    .start_input_with_prompt("Save As", InputAction::SaveAs);
                                renderer::render(&file_manager);
                            }
                            _ => {}
                        }
                    }
                }
                0x13 => {
                    // Ctrl+S to save
                    match file_manager.save() {
                        Ok(_) => {
                            file_manager.add_toast(
                                "File saved successfully!",
                                3000,
                                nox_editor::ToastType::Success,
                            );
                        }
                        Err(e) => {
                            file_manager.add_toast(
                                &format!("Error saving file: {}", e),
                                5000,
                                nox_editor::ToastType::Error,
                            );
                        }
                    }
                }
                0x11 => break,                   // Ctrl+Q to quit
                0x0a => file_manager.new_line(), // Line Feed (LF) - Unix style
                0x0d => file_manager.new_line(), // Carriage Return (CR) - Mac style
                0x09 => file_manager.tab(),
                key => {
                    // Handle other keys
                    if key.is_ascii_graphic() || key == b' ' {
                        file_manager.insert_char(key as char);
                    } else if key == 0x7f || key == 0x08 {
                        file_manager.delete_char();
                    }
                }
            }
            renderer::render(&file_manager);
        }
    }

    //Exit code
    disable_terminal_raw_mode().expect("Failed to disable raw mode");
    clear_screen();
}
