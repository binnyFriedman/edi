use std::io;
use std::os::unix::io::{AsRawFd, RawFd};
use termios::*;
use termios::os::target::IUTF8;

#[derive(Debug)]
pub struct TermiosWrap {
    fd: RawFd,
    original_termios: Termios,
}

impl TermiosWrap {
    pub fn new()-> Self {
        let stdin_fd = io::stdin().as_raw_fd();
        let original_termios = Termios::from_fd(stdin_fd).unwrap();
        let r#override = Self::r#override(original_termios);
        tcsetattr(stdin_fd,TCSAFLUSH,&r#override).unwrap();
        TermiosWrap {
            fd: stdin_fd,
            original_termios,
        }
    }
    fn r#override(mut termios: Termios)->Termios{
        termios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        termios.c_oflag &= !(OPOST);
        termios.c_cflag |= CS8;
        termios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
        termios.c_cc[VMIN] = 0;
        termios.c_cc[VTIME] = 1;
        //enable utf8
        termios.c_iflag &= IUTF8;

        termios
    }
}

impl Drop for TermiosWrap {
    fn drop(&mut self) {
        tcsetattr(self.fd,TCSAFLUSH,&self.original_termios).unwrap();
    }
}

impl Default for TermiosWrap {
    fn default()->Self {
        Self::new()
    }
}
