use libc::{TIOCGWINSZ, ioctl, winsize};
use std::io::{self, stdin};
use std::{io::Write, os::unix::io::AsRawFd};
use termios::{ECHO, ICANON, ISIG, IXON, TCSANOW, Termios, tcsetattr};

// Export the colors module
pub mod colors;

pub fn set_terminal_raw_mode() -> io::Result<()> {
    let fd = stdin().as_raw_fd();
    let mut termios = Termios::from_fd(fd)?;
    // Disable canonical mode, echo, signal handling, and software flow control
    termios.c_lflag &= !(ICANON | ECHO | ISIG);
    termios.c_iflag &= !IXON;

    tcsetattr(fd, TCSANOW, &termios)?;
    Ok(())
}

pub fn disable_terminal_raw_mode() -> io::Result<()> {
    let fd = stdin().as_raw_fd();
    let mut termios = Termios::from_fd(fd)?;
    // Restore canonical mode, echo, signal handling, and software flow control
    termios.c_lflag |= ICANON | ECHO | ISIG;
    termios.c_iflag |= IXON;
    tcsetattr(fd, TCSANOW, &termios)?;
    Ok(())
}

pub fn clear_screen() {
    if let Ok(output) = std::process::Command::new("clear").output() {
        if output.status.success() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            print!("\x1B[2J\x1B[1;1H");
        }
    } else {
        print!("\x1B[2J\x1B[1;1H");
    }
    io::stdout().flush().expect("Failed to clear screen");
}

pub fn get_terminal_size() -> io::Result<(u16, u16)> {
    let mut ws = winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let fd = io::stdout().as_raw_fd();

    if unsafe { ioctl(fd, TIOCGWINSZ, &mut ws) } == -1 {
        return Err(io::Error::last_os_error());
    }

    Ok((ws.ws_row, ws.ws_col))
}
