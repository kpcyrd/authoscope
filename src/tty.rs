// this whole file is vendored until https://github.com/a8m/pb/pull/62 is fixed

extern crate libc;

#[derive(Debug)]
pub struct Width(pub u16);
#[derive(Debug)]
pub struct Height(pub u16);

// No-op on any other operating system.
#[cfg(not(any(target_os = "dragonfly", target_os = "freebsd")))]
fn ioctl_conv<T: Copy>(v: T) -> T { v }

/// Returns the size of the terminal, if available.
///
/// If STDOUT is not a tty, returns `None`
pub fn terminal_size() -> Option<(Width, Height)> {
    use self::libc::{ioctl, isatty, STDOUT_FILENO, TIOCGWINSZ, winsize};
    let is_tty: bool = unsafe { isatty(STDOUT_FILENO) == 1 };

    if !is_tty {
        return None;
    }

    let (rows, cols) = unsafe {
        let mut winsize = winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        ioctl(STDOUT_FILENO, ioctl_conv(TIOCGWINSZ), &mut winsize);
        let rows = if winsize.ws_row > 0 {
            winsize.ws_row
        } else {
            0
        };
        let cols = if winsize.ws_col > 0 {
            winsize.ws_col
        } else {
            0
        };
        (rows as u16, cols as u16)
    };

    if rows > 0 && cols > 0 {
        Some((Width(cols), Height(rows)))
    } else {
        None
    }
}
