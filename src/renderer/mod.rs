use std::io::{Write, stdout};

use nox_editor::{FileManager, InputAction, ToastType};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

use crate::utils::colors::*;
use crate::utils::{clear_screen, get_terminal_size};

// Static for syntax/theme sets (load once)
lazy_static::lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
}

fn center_text(text: &str, width: usize) -> String {
    if text.len() >= width {
        return text.to_string();
    }

    let padding = (width - text.len()) / 2;
    format!("{}{}", " ".repeat(padding), text)
}

pub fn render(file_manager: &FileManager) {
    clear_screen();

    //TODO: Improve this function
    if (file_manager.file_browser.browser_open) {
        render_browser(file_manager);
        return;
    }

    let (terminal_rows, terminal_cols) = get_terminal_size().unwrap_or((24, 80));

    // Calculate used lines BEFORE rendering content
    let mut used_lines = 4; // title bar (2) + status bar (1) + separator (1)

    if file_manager.input_handler.taking_input {
        used_lines += 2; // input prompt + help
    } else if !file_manager.toasts.is_empty() {
        used_lines += 1; // toast message
    }

    let show_footer = terminal_rows > used_lines + 1;
    if show_footer {
        used_lines += 1; // footer
    }

    render_title_bar(file_manager, terminal_cols);
    render_content(file_manager, terminal_rows, used_lines);
    render_status_bar(file_manager, terminal_cols);

    // Only show toasts if not taking input
    if file_manager.input_handler.taking_input {
        render_input_prompt(file_manager);
    } else if !file_manager.toasts.is_empty() {
        render_toasts(file_manager);
    }

    if show_footer {
        render_footer();
    }

    stdout().flush().expect("Failed to flush stdout");
}

fn render_title_bar(file_manager: &FileManager, terminal_cols: u16) {
    let filename = if file_manager.file_info.name.is_empty() {
        "Untitled".to_string()
    } else {
        file_manager.file_info.name.clone()
    };

    //scroll position
    let buffer_len = file_manager.buffer.data.len();
    let current_pos = file_manager.pointer.y;
    let scroll_indicator = if buffer_len > 0 {
        let percentage = (current_pos * 100) / std::cmp::max(1, buffer_len - 1);
        format!(" [{}%]", percentage)
    } else {
        "".to_string()
    };

    let term_cols_usize = terminal_cols as usize;

    let display_title = format!("{}{}", filename, scroll_indicator);

    let mut title_line = String::new();
    title_line.push_str(BG_BLUE);
    title_line.push_str(BRIGHT_WHITE);
    title_line.push_str(BOLD);

    for _ in 0..term_cols_usize {
        title_line.push(' ');
    }

    println!(
        "{}\r{}{}{}{}",
        title_line,
        BG_BLUE,
        BRIGHT_WHITE,
        BOLD,
        center_text(&display_title, term_cols_usize)
    );

    println!("{}{}", RESET, "═".repeat(term_cols_usize));
}

fn highlight_line(line: &str, extension: &str) -> String {
    let syntax = match extension {
        //Cached Lookup for common file types
        "rs" => SYNTAX_SET.find_syntax_by_extension("rs"),
        "js" => SYNTAX_SET.find_syntax_by_extension("js"),
        "py" => SYNTAX_SET.find_syntax_by_extension("py"),
        "md" => SYNTAX_SET.find_syntax_by_extension("md"),
        "txt" => Some(SYNTAX_SET.find_syntax_plain_text()),
        "" => Some(SYNTAX_SET.find_syntax_plain_text()),
        _ => SYNTAX_SET.find_syntax_by_extension(extension),
    }
    .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, &THEME_SET.themes["base16-ocean.dark"]);

    //Converts to terminal color codes
    match highlighter.highlight_line(line, &SYNTAX_SET) {
        Ok(ranges) => as_24_bit_terminal_escaped(&ranges, false),
        Err(_) => line.to_string(),
    }
}

fn render_content(file_manager: &FileManager, terminal_rows: u16, used_lines: u16) {
    // Calculate max content lines from the passed used_lines
    let max_content_lines = terminal_rows.saturating_sub(used_lines) as usize;
    let max_content_lines = std::cmp::max(1, max_content_lines);
    let current_line = file_manager.pointer.y;
    let buffer_line_count = file_manager.buffer.data.len();
    if buffer_line_count == 0 {
        println!("{}     [Empty buffer]{}", DIM, RESET);
        return;
    }
    let half_height = max_content_lines / 2;
    let start_line = if current_line > half_height {
        if current_line + half_height > buffer_line_count {
            let start = buffer_line_count.saturating_sub(max_content_lines);
            std::cmp::min(start, buffer_line_count - 1)
        } else {
            current_line.saturating_sub(half_height)
        }
    } else {
        0
    };
    let end_line = std::cmp::min(start_line + max_content_lines, buffer_line_count);

    // Get file extension for syntax
    let extension = file_manager.file_info.name.split('.').last().unwrap_or("");

    //Render the content
    for i in start_line..end_line {
        let line = &file_manager.buffer.data[i];

        let is_current_line = i == file_manager.pointer.y;

        let line_num_style = if is_current_line {
            format!("{}{}", BOLD, BRIGHT_CYAN)
        } else {
            format!("{}", BRIGHT_BLACK)
        };

        let highlighted = highlight_line(line, extension);

        if is_current_line {
            //Line with the cursor visible
            let cursor_line = if line.is_empty() {
                format!("{}{}{}", REVERSE, " ", RESET)
            } else {
                let char_count = line.chars().count();
                //Cursor at the last
                if file_manager.pointer.x >= char_count {
                    format!("{}{}{}{}", highlighted, REVERSE, " ", RESET)
                } else {
                    // Insert cursor at the right position
                    let mut result = String::new();
                    let mut char_index = 0;

                    let mut in_escape = false;
                    let mut escape_sequence = String::new();

                    for c in highlighted.chars() {
                        if in_escape {
                            escape_sequence.push(c);
                            if c == 'm' {
                                result.push_str(&escape_sequence);
                                escape_sequence.clear();
                                in_escape = false;
                            }
                        } else if c == '\x1b' {
                            escape_sequence.push(c);
                            in_escape = true;
                        } else {
                            if char_index == file_manager.pointer.x {
                                result.push_str(REVERSE);
                                result.push(c);
                                result.push_str(RESET);
                            } else {
                                result.push(c);
                            }
                            char_index += 1;
                        }
                    }
                    result
                }
            };

            println!("{}{:3}{}│ {}", line_num_style, i + 1, RESET, cursor_line);
        } else {
            println!(
                "{}{:3}{}│ {}{}",
                line_num_style,
                i + 1,
                RESET,
                highlighted,
                RESET
            );
        }
    }

    let visible_lines = (end_line - start_line) as u16;
    let max_visible_lines = terminal_rows.saturating_sub(used_lines);
    if visible_lines < max_visible_lines {
        let blank_lines = max_visible_lines - visible_lines;
        for _ in 0..blank_lines {
            println!(" ");
        }
    }
}

fn render_status_bar(file_manager: &FileManager, terminal_cols: u16) {
    let current_line = file_manager.pointer.y + 1;
    let total_lines = file_manager.buffer.data.len();
    let cursor_pos = format!(
        "Line: {}/{}, Col: {}",
        current_line,
        total_lines,
        file_manager.pointer.x + 1
    );

    let file_info = format!("File: {}", file_manager.file_info.path);

    let cols_usize = terminal_cols as usize;
    let padding_size = cols_usize
        .saturating_sub(file_info.len())
        .saturating_sub(cursor_pos.len())
        .saturating_sub(2); // 2 spaces for separation
    let padding = " ".repeat(padding_size);

    println!(
        "{}{}{}{}{}{}{}",
        BG_BLACK, BRIGHT_WHITE, file_info, padding, cursor_pos, " ", RESET
    );
}

fn render_input_prompt(file_manager: &FileManager) {
    if file_manager.input_handler.taking_input {
        let (_, terminal_cols) = get_terminal_size().unwrap_or((24, 80));
        let term_width = terminal_cols as usize;

        let input_text = format!(
            "{}: {}{}",
            file_manager.input_handler.input_prompt, file_manager.input_handler.input_buffer, "■"
        );

        let display_text = if input_text.len() > term_width - 2 {
            format!(" {}...", &input_text[..term_width - 6])
        } else {
            let padding = term_width
                .saturating_sub(input_text.len())
                .saturating_sub(1);
            format!(" {}{}", input_text, " ".repeat(padding))
        };

        let (bg_color, fg_color) = match file_manager.input_handler.action_type {
            InputAction::SaveAs => (BG_BLUE, BRIGHT_WHITE),
            _ => (BG_CYAN, BRIGHT_WHITE),
        };

        println!("{}{}{}{}{}", bg_color, fg_color, BOLD, display_text, RESET);

        println!(
            "{}{}ESC{} cancel │ {}{}ENTER{} confirm{}",
            BRIGHT_BLACK, BOLD, RESET, BOLD, BRIGHT_WHITE, RESET, BRIGHT_BLACK
        );
    }
}

fn render_toasts(file_manager: &FileManager) {
    if file_manager.toasts.is_empty() {
        return;
    }

    if let Some(toast) = file_manager.toasts.last() {
        let (bg_color, fg_color, prefix) = match toast.toast_type {
            ToastType::Info => (BG_BLUE, BRIGHT_WHITE, "ℹ"),
            ToastType::Success => (BG_GREEN, BLACK, "✓"),
            ToastType::Warning => (BG_YELLOW, BLACK, "!"),
            ToastType::Error => (BG_RED, BRIGHT_WHITE, "✗"),
        };

        let (_, terminal_cols) = get_terminal_size().unwrap_or((24, 80));
        let term_width = terminal_cols as usize;

        let count_text = if file_manager.toasts.len() > 1 {
            format!(" (+{} more)", file_manager.toasts.len() - 1)
        } else {
            "".to_string()
        };

        let message_text = format!("{} {}{}", prefix, toast.message, count_text);

        // Left-align the toast with some padding
        let padding_needed = term_width
            .saturating_sub(message_text.len())
            .saturating_sub(2);
        let toast_line = format!(" {}{}", message_text, " ".repeat(padding_needed + 1));

        println!("{}{}{}{}", bg_color, fg_color, toast_line, RESET);
    }
}

fn render_footer() {
    let shortcuts = [
        ("Ctrl+S", "Save"),
        ("Alt+S", "Save As"),
        ("Ctrl+Q", "Exit"),
        ("ESC", "Cancel Input"),
        ("↑/↓/←/→", "Navigate"),
    ];

    print!("{}", BRIGHT_BLACK);

    for (i, (key, action)) in shortcuts.iter().enumerate() {
        if i > 0 {
            print!(" │ ");
        }
        print!("{}{}{}{} {}", BRIGHT_WHITE, BOLD, key, RESET, action);
    }

    println!("{}", RESET);
}

pub fn render_browser(fm: &FileManager) {
    clear_screen();

    let (terminal_rows, terminal_cols) = get_terminal_size().unwrap_or((24, 80));
    let term_cols_usize = terminal_cols as usize;

    // Title bar for file browser
    let mut title_line = String::new();
    title_line.push_str(BG_BLUE);
    title_line.push_str(BRIGHT_WHITE);
    title_line.push_str(BOLD);

    for _ in 0..term_cols_usize {
        title_line.push(' ');
    }

    println!(
        "{}\r{}{}{}{}",
        title_line,
        BG_BLUE,
        BRIGHT_WHITE,
        BOLD,
        center_text("📁 File Browser", term_cols_usize)
    );

    println!("{}{}", RESET, "═".repeat(term_cols_usize));

    match &fm.file_browser.paths {
        Some(files) => {
            if files.is_empty() {
                println!("{}     [Empty directory]{}", DIM, RESET);
                println!();
                println!("{}{}ESC{} to go back", BRIGHT_BLACK, BOLD, RESET);
                return;
            }

            // Calculate available space for file list
            let used_lines = 6; // title (2) + separator (1) + help (3)
            let max_files = (terminal_rows as usize).saturating_sub(used_lines);

            // Calculate scroll position
            let current_pos = fm.file_browser.pointer;
            let start_index = if current_pos >= max_files {
                current_pos.saturating_sub(max_files / 2)
            } else {
                0
            };
            let end_index = std::cmp::min(start_index + max_files, files.len());

            // Show scroll indicator if needed
            if files.len() > max_files {
                let percentage = (current_pos * 100) / std::cmp::max(1, files.len() - 1);
                println!(
                    "{}Showing {}-{} of {} files [{}%]{}",
                    DIM,
                    start_index + 1,
                    end_index,
                    files.len(),
                    percentage,
                    RESET
                );
            } else {
                println!("{}Showing {} files{}", DIM, files.len(), RESET);
            }

            for i in start_index..end_index {
                let entry = &files[i];
                let is_selected = i == fm.file_browser.pointer;
                let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                let file_name = entry.file_name().to_string_lossy().to_string();

                let (icon, name_color) = if is_dir {
                    ("📁", BRIGHT_CYAN)
                } else {
                    let ext = file_name.split('.').last().unwrap_or("");
                    match ext {
                        "rs" => ("🦀", BRIGHT_YELLOW),
                        "js" | "ts" => ("📜", BRIGHT_YELLOW),
                        "py" => ("🐍", BRIGHT_GREEN),
                        "md" => ("📝", BRIGHT_WHITE),
                        "txt" => ("📄", BRIGHT_WHITE),
                        "json" => ("⚙️", BRIGHT_MAGENTA),
                        "toml" | "yaml" | "yml" => ("⚙️", BRIGHT_BLUE),
                        _ => ("📄", BRIGHT_WHITE),
                    }
                };

                if is_selected {
                    print!("{}{}{}", BG_CYAN, BLACK, BOLD);
                    let padding = term_cols_usize.saturating_sub(file_name.len() + 4);
                    println!("► {} {}{}{}", icon, file_name, " ".repeat(padding), RESET);
                } else {
                    println!("  {}{} {}{}", icon, name_color, file_name, RESET);
                }
            }

            let shown_files = end_index - start_index;
            let remaining_lines = max_files.saturating_sub(shown_files);
            for _ in 0..remaining_lines {
                println!();
            }
        }
        None => {
            println!("{}     [No files found]{}", DIM, RESET);
            println!();
        }
    }

    println!("{}{}", RESET, "─".repeat(term_cols_usize));
    println!(
        "{}{}↑/↓{} Navigate │ {}{}ENTER{} Open/Select │ {}{}ESC{} Close Browser{}",
        BRIGHT_BLACK,
        BOLD,
        RESET,
        BOLD,
        BRIGHT_WHITE,
        RESET,
        BOLD,
        BRIGHT_WHITE,
        RESET,
        BRIGHT_BLACK
    );

    stdout().flush().expect("Failed to flush stdout");
}
