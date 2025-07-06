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
        let file_result = open_file(&args[1]);
        match file_result {
            Ok(content) => data = content,
            Err(e) => {
            eprintln!("Error opening file: {}", e);
            data = vec![String::new()];
            }
        }
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

        if file_manager.file_browser.browser_open {
            if stdin.read(&mut buffer).is_ok() {
                match buffer[0] {
                    // Handle Arrow Keys and Escape
                    b'\x1b' => {
                        let mut arrow_buffer = [0; 2];
                        if stdin.read(&mut arrow_buffer).is_ok() {
                            match arrow_buffer {
                                [b'[', b'A'] => file_manager.file_browser.move_pointer(-1), // Up arrow - decrease pointer
                                [b'[', b'B'] => file_manager.file_browser.move_pointer(1), // Down arrow - increase pointer
                                _ => {
                                    // This might be just an escape key
                                    file_manager.file_browser.close_browser();
                                    file_manager.add_toast(
                                        "File browser closed",
                                        2000,
                                        nox_editor::ToastType::Info,
                                    );
                                }
                            }
                        } else {
                            // Single escape key press
                            file_manager.file_browser.close_browser();
                            file_manager.add_toast(
                                "File browser closed",
                                2000,
                                nox_editor::ToastType::Info,
                            );
                        }
                    }
                    // Enter key - open selected file/directory
                    0x0d | 0x0a => {
                        if let Some(entry) = file_manager.file_browser.get_selected_entry() {
                            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

                            if is_dir {
                                // Navigate into directory
                                let path = entry.path().to_string_lossy().to_string();
                                match file_manager.file_browser.open_browser(&path) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        file_manager.add_toast(
                                            &e,
                                            5000,
                                            nox_editor::ToastType::Error,
                                        );
                                    }
                                }
                            } else {
                                // Open file - replace current buffer
                                let path = entry.path().to_string_lossy().to_string();
                                match fm::open_file(&path) {
                                    Ok(new_data) => {
                                        file_manager.buffer.data = if new_data.is_empty() {
                                            vec![String::new()]
                                        } else {
                                            new_data
                                        };
                                        file_manager.file_info.path = path.clone();
                                        file_manager.file_info.name =
                                            path.split('/').last().unwrap_or("unknown").to_string();
                                        file_manager.pointer.x = 0;
                                        file_manager.pointer.y = 0;
                                        file_manager.file_browser.close_browser();
                                        file_manager.add_toast(
                                            &format!("Opened: {}", file_manager.file_info.name),
                                            3000,
                                            nox_editor::ToastType::Success,
                                        );
                                    }
                                    Err(e) => {
                                        file_manager.add_toast(
                                            &format!("Error opening file: {}", e),
                                            5000,
                                            nox_editor::ToastType::Error,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
                renderer::render(&file_manager);
            }
            continue; // Don't process other inputs while browser is open
        }

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
                //CTRL+O to open file
                0x0f => {
                    // Open file dialog - start from current working directory or file's directory
                    let start_path = if file_manager.file_info.path == "/"
                        || file_manager.file_info.path.is_empty()
                    {
                        std::env::current_dir()
                            .unwrap_or_else(|_| std::path::PathBuf::from("."))
                            .to_string_lossy()
                            .to_string()
                    } else {
                        // Use the directory of the current file
                        std::path::Path::new(&file_manager.file_info.path)
                            .parent()
                            .unwrap_or_else(|| std::path::Path::new("."))
                            .to_string_lossy()
                            .to_string()
                    };

                    match file_manager.file_browser.open_browser(&start_path) {
                        Ok(_) => {
                            file_manager.add_toast(
                                "File browser opened - Use ↑/↓ to navigate, Enter to select, ESC to close",
                                4000,
                                nox_editor::ToastType::Info,
                            );
                        }
                        Err(e) => {
                            file_manager.add_toast(&e, 5000, nox_editor::ToastType::Error);
                        }
                    }
                    renderer::render(&file_manager);
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
