// use getch;
use getch::Getch;

#[cfg(not(windows))]
use termios::{self, tcsetattr, ICANON, ECHO};

pub struct Keyboard {
    getch: Getch,
}

impl Keyboard {
    pub fn new() -> Keyboard {
        let getch = Getch::new();

        Keyboard {
            getch,
        }
    }

    pub fn get(&self) -> Key {
        loop {
            let key = self.getch.getch();
            // println!("{:?}", key);
            match key {
                Ok(112) => return Key::P,
                Ok(114) => return Key::R,
                Ok(43)  => return Key::Plus,
                Ok(45)  => return Key::Minus,
                _ => (),
            }
        }
    }

    // since the getch thread is orphaned, we have to cleanup manually
    pub fn reset() {
        #[cfg(not(windows))]
        {
            if let Ok(mut termios) = termios::Termios::from_fd(0) {
                termios.c_lflag |= ICANON|ECHO;
                tcsetattr(0,termios::TCSADRAIN, &termios).unwrap_or(());
            }
        }
    }
}

#[derive(Debug)]
pub enum Key {
    P,
    R,
    Plus,
    Minus,
}
