use std::fs::{self, DirEntry};
use std::io::Write;
use std::time::{Duration, Instant};

pub struct Pointer {
    pub x: usize,
    pub y: usize,
}

pub struct Buffer {
    pub data: Vec<String>,
    pub current_line: usize,
}

// Toast notification system
pub struct Toast {
    pub message: String,
    pub created_at: Instant,
    pub duration: Duration,
    pub toast_type: ToastType,
}

#[derive(PartialEq)]
pub enum ToastType {
    Info,
    Success,
    Warning,
    Error,
}

impl Toast {
    pub fn new(message: &str, duration_ms: u64, toast_type: ToastType) -> Self {
        Toast {
            message: message.to_string(),
            created_at: Instant::now(),
            duration: Duration::from_millis(duration_ms),
            toast_type,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }
}

impl Buffer {
    pub fn new(data: Vec<String>) -> Self {
        Buffer {
            data,
            current_line: 0,
        }
    }
}

impl Pointer {
    pub fn new(x: usize, y: usize) -> Self {
        Pointer { x, y }
    }
}

pub struct FileInfo {
    pub name: String,
    pub path: String,
}

// Input action types
#[derive(Clone, Copy, PartialEq)]
pub enum InputAction {
    Generic,
    SaveAs,
    // Add more action types as needed
}

// Result of input handling
pub enum InputResult {
    Confirmed(String),
    Cancelled,
    InProgress,
}

pub struct InputHandler {
    pub taking_input: bool,
    pub input_buffer: String,
    pub input_prompt: String,
    pub action_type: InputAction,
}

impl InputHandler {
    pub fn new() -> Self {
        InputHandler {
            taking_input: false,
            input_buffer: String::new(),
            input_prompt: "Input: ".to_string(),
            action_type: InputAction::Generic,
        }
    }

    pub fn start_input(&mut self) {
        self.taking_input = true;
        self.input_buffer.clear();
        self.action_type = InputAction::Generic;
    }

    pub fn start_input_with_prompt(&mut self, prompt: &str, action: InputAction) {
        self.taking_input = true;
        self.input_buffer.clear();
        self.input_prompt = prompt.to_string();
        self.action_type = action;
    }

    pub fn confirm_input(&mut self) -> InputResult {
        self.taking_input = false;
        InputResult::Confirmed(std::mem::take(&mut self.input_buffer))
    }

    pub fn cancel_input(&mut self) -> InputResult {
        self.taking_input = false;
        self.input_buffer.clear();
        InputResult::Cancelled
    }

    pub fn add_char(&mut self, c: char) {
        if self.taking_input {
            self.input_buffer.push(c);
        }
    }

    pub fn delete_char(&mut self) {
        if self.taking_input {
            self.input_buffer.pop();
        }
    }

    // Handle a key press during input
    pub fn handle_key(&mut self, key: u8) -> InputResult {
        if !self.taking_input {
            return InputResult::InProgress;
        }

        match key {
            0x0d => self.confirm_input(), // Enter key
            0x0a => self.confirm_input(), // Enter key
            0x1b => self.cancel_input(),  // Escape key
            0x08 | 0x7f => {
                // Backspace or Delete
                self.delete_char();
                InputResult::InProgress
            }
            key if key.is_ascii_graphic() || key == b' ' => {
                self.add_char(key as char);
                InputResult::InProgress
            }
            _ => {
                if let Some(c) = char::from_u32(key as u32) {
                    if c.is_ascii_graphic() || c == ' ' {
                        self.add_char(c);
                    }
                }
                InputResult::InProgress
            }
        }
    }
}

pub struct FileManager {
    pub pointer: Pointer,
    pub buffer: Buffer,
    pub file_info: FileInfo,
    pub toasts: Vec<Toast>,
    pub input_handler: InputHandler,
    pub file_browser: FileBrowser,
}

impl FileManager {
    pub fn new(buffer: Buffer, file_info: FileInfo) -> Self {
        let pointer = Pointer::new(0, 0);
        FileManager {
            pointer,
            buffer,
            file_info,
            toasts: Vec::new(),
            input_handler: InputHandler::new(),
            file_browser: FileBrowser::new(),
        }
    }

    pub fn add_toast(&mut self, message: &str, duration_ms: u64, toast_type: ToastType) {
        let toast = Toast::new(message, duration_ms, toast_type);
        self.toasts.push(toast);
    }

    pub fn update_toasts(&mut self) {
        self.toasts.retain(|toast| !toast.is_expired());
    }

    pub fn move_pointer(&mut self, dy: isize, dx: isize) {
        let new_y = if dy < 0 {
            self.pointer.y.saturating_sub(dy.abs() as usize)
        } else {
            self.pointer.y.saturating_add(dy as usize)
        };

        let max_y = self.buffer.data.len().saturating_sub(1);
        let bounded_y = std::cmp::min(new_y, max_y);

        let new_x = if dx < 0 {
            self.pointer.x.saturating_sub(dx.abs() as usize)
        } else {
            self.pointer.x.saturating_add(dx as usize)
        };

        let current_line_len = self.buffer.data.get(bounded_y).map_or(0, |line| line.len());

        let bounded_x = std::cmp::min(new_x, current_line_len);

        self.pointer.y = bounded_y;
        self.pointer.x = bounded_x;
    }

    pub fn insert_char(&mut self, c: char) {
        if let Some(line) = self.buffer.data.get_mut(self.pointer.y) {
            let safe_x = std::cmp::min(self.pointer.x, line.len());
            line.insert(safe_x, c);
            self.pointer.x += 1;
        }
    }
    pub fn delete_char(&mut self) {
        if self.pointer.x > 0 && self.pointer.y < self.buffer.data.len() {
            if let Some(line) = self.buffer.data.get_mut(self.pointer.y) {
                line.remove(self.pointer.x - 1);
                self.pointer.x -= 1;
            }
        } else if self.pointer.y > 0 && self.pointer.x == 0 {
            let current_line = self.buffer.data.remove(self.pointer.y);
            self.pointer.y -= 1;

            if let Some(prev_line) = self.buffer.data.get_mut(self.pointer.y) {
                self.pointer.x = prev_line.len();
                prev_line.push_str(&current_line);
            }
        }
    }

    pub fn new_line(&mut self) {
        if self.pointer.x
            < self
                .buffer
                .data
                .get(self.pointer.y)
                .map_or(0, |line| line.len())
        {
            let mut current_line = self.buffer.data.remove(self.pointer.y);
            let new_line = current_line.split_off(self.pointer.x);
            self.buffer.data.insert(self.pointer.y, current_line);
            self.buffer.data.insert(self.pointer.y + 1, new_line);
        } else {
            self.buffer.data.insert(self.pointer.y + 1, String::new());
        }
        self.pointer.y += 1;
        self.pointer.x = 0;
    }

    pub fn save(&self) -> std::io::Result<()> {
        let mut file = std::fs::File::create(&self.file_info.path)?;
        for line in &self.buffer.data {
            writeln!(file, "{}", line)?;
        }
        Ok(())
    }

    pub fn save_as(&mut self, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        for line in &self.buffer.data {
            writeln!(file, "{}", line)?;
        }
        self.file_info.path = path.to_string();
        self.file_info.name = path.split('/').last().unwrap_or("unknown").to_string();
        Ok(())
    }

    pub fn tab(&mut self) {
        let spaces = "    "; // 4 spaces for a tab
        if let Some(line) = self.buffer.data.get_mut(self.pointer.y) {
            let safe_x = std::cmp::min(self.pointer.x, line.len());
            line.insert_str(safe_x, spaces);
            self.pointer.x += spaces.len();
        }
    }

    pub fn remove_current_line(&mut self) {
        if self.pointer.y < self.buffer.data.len() {
            self.buffer.data.remove(self.pointer.y);
            if self.pointer.y >= self.buffer.data.len() {
                self.pointer.y = self.buffer.data.len().saturating_sub(1);
            }
            self.pointer.x = 0;
        }
    }
}

pub struct FileBrowser {
    pub browser_open: bool,
    pub pointer: usize,
    pub paths: Option<Vec<DirEntry>>,
}

impl FileBrowser {
    pub fn new() -> Self {
        FileBrowser {
            browser_open: false,
            pointer: 0,
            paths: None,
        }
    }

    pub fn open_browser(&mut self, path: &str) -> Result<(), String> {
        let entries = fs::read_dir(path);
        let mut files: Vec<DirEntry> = vec![];

        match entries {
            Err(e) => {
                return Err(format!("Error opening file browser: {}", e));
            }
            Ok(files_iterator) => {
                for i in files_iterator {
                    match i {
                        Ok(entry) => files.push(entry),
                        Err(e) => return Err(format!("Error reading directory entry: {}", e)),
                    }
                }
            }
        }

        // Sort files: directories first, then files alphabetically
        files.sort_by(|a, b| {
            let a_is_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let b_is_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        self.paths = Some(files);
        self.browser_open = true;
        self.pointer = 0;
        Ok(())
    }

    pub fn move_pointer(&mut self, y: i8) {
        if let Some(paths) = self.paths.as_ref() {
            if paths.is_empty() {
                return;
            }

            if y < 0 {
                // Up arrow - decrease pointer
                self.pointer = self.pointer.saturating_sub(1);
            } else if y > 0 {
                // Down arrow - increase pointer
                if self.pointer < paths.len().saturating_sub(1) {
                    self.pointer += 1;
                }
            }
        }
    }

    pub fn close_browser(&mut self) {
        self.browser_open = false;
        self.pointer = 0;
        self.paths = None;
    }

    pub fn get_selected_path(&self) -> Option<String> {
        if let Some(paths) = &self.paths {
            if self.pointer < paths.len() {
                return Some(paths[self.pointer].path().to_string_lossy().to_string());
            }
        }
        None
    }

    pub fn get_selected_entry(&self) -> Option<&DirEntry> {
        if let Some(paths) = &self.paths {
            if self.pointer < paths.len() {
                return Some(&paths[self.pointer]);
            }
        }
        None
    }
}
