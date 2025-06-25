// ANSI color codes
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const ITALIC: &str = "\x1b[3m";
pub const UNDERLINE: &str = "\x1b[4m";
pub const BLINK: &str = "\x1b[5m";
pub const REVERSE: &str = "\x1b[7m";
pub const HIDDEN: &str = "\x1b[8m";

// Foreground colors
pub const BLACK: &str = "\x1b[38;5;235m"; // #1a1b26
pub const RED: &str = "\x1b[38;5;204m"; // #f7768e
pub const GREEN: &str = "\x1b[38;5;114m"; // #9ece6a
pub const YELLOW: &str = "\x1b[38;5;180m"; // #e0af68
pub const BLUE: &str = "\x1b[38;5;111m"; // #7aa2f7
pub const MAGENTA: &str = "\x1b[38;5;141m"; // #bb9af7
pub const CYAN: &str = "\x1b[38;5;117m"; // #7dcfff
pub const WHITE: &str = "\x1b[38;5;145m"; // #a9b1d6
pub const BRIGHT_BLACK: &str = "\x1b[38;5;238m"; // #414868
pub const BRIGHT_RED: &str = "\x1b[38;5;204m"; // #f7768e
pub const BRIGHT_GREEN: &str = "\x1b[38;5;114m"; // #9ece6a
pub const BRIGHT_YELLOW: &str = "\x1b[38;5;180m"; // #e0af68
pub const BRIGHT_BLUE: &str = "\x1b[38;5;111m"; // #7aa2f7
pub const BRIGHT_MAGENTA: &str = "\x1b[38;5;141m"; // #bb9af7
pub const BRIGHT_CYAN: &str = "\x1b[38;5;117m"; // #7dcfff
pub const BRIGHT_WHITE: &str = "\x1b[38;5;189m"; // #c0caf5

// Background colors
pub const BG_BLACK: &str = "\x1b[48;5;235m"; // #1a1b26
pub const BG_RED: &str = "\x1b[48;5;204m"; // #f7768e
pub const BG_GREEN: &str = "\x1b[48;5;114m"; // #9ece6a
pub const BG_YELLOW: &str = "\x1b[48;5;180m"; // #e0af68
pub const BG_BLUE: &str = "\x1b[48;5;111m"; // #7aa2f7
pub const BG_MAGENTA: &str = "\x1b[48;5;141m"; // #bb9af7
pub const BG_CYAN: &str = "\x1b[48;5;117m"; // #7dcfff
pub const BG_WHITE: &str = "\x1b[48;5;145m"; // #a9b1d6

// Toasts and status bar (using blue and magenta backgrounds for highlight)
pub const TOAST_BG: &str = "\x1b[48;5;60m"; // #3d59a1 (Tokyo Night blue)
pub const TOAST_FG: &str = "\x1b[38;5;189m"; // #c0caf5 (bright white)
pub const STATUS_BG: &str = "\x1b[48;5;60m"; // #3d59a1 (Tokyo Night blue)
pub const STATUS_FG: &str = "\x1b[38;5;189m"; // #c0caf5 (bright white)
