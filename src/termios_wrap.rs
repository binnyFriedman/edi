use std::io;
use std::io::{Read, stdout, Write};
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

pub fn get_window_size()->(u16,u16){
    print!("{}{}",term_sequence(TCC::CRight(999)),term_sequence(TCC::CDown(999)));
    stdout().flush().unwrap();
    get_cursor_position()
}

pub  fn get_cursor_position()->(u16,u16){
    let mut pos = String::new();
    print!("{}",term_sequence(TCC::Cpr, ));
    io::stdout().flush().unwrap();
    //read the cursor pos until the 'R'
    let mut index = 0;
    loop {
        let mut buf = [0;1];
        let bytes_read =io::stdin().read(&mut buf).unwrap();
        if bytes_read == 0{
            continue;
        }
        if buf[0] == b'R' {
            break;
        }
        if index > 1 {
            pos.push(buf[0] as char);
        }
        index += 1;
    }
    //slice the pos string
    let mut iter = pos.split(';');
    let rows = iter.next().unwrap().parse::<u16>().unwrap();
    let cols = iter.next().unwrap().parse::<u16>().unwrap();


    (rows,cols)
}
// Terminal Control Commands
pub enum TCC{
    Cpr,//Cursor Position Report
    CLeft(usize),//Cursor Backward
    CDown(usize),//Cursor Down
    CRight(usize),//Cursor Forward
    PosC(usize, usize),//Cursor Position
    CUp(usize),//Cursor Up

    EraseDisplay(Erase),//Erase Display
    EraseLine(Erase),//Erase Line

    Hide,//Hide Cursor
    Show,//Show Cursor


}
pub enum Erase{
    FromCursor,
    ToCursor,
    All,
}

fn resolve_erase(erase:Erase)->String{
    match erase {
        Erase::FromCursor => "0",
        Erase::ToCursor => "1",
        Erase::All => "2",
    }.to_string()
}

pub fn term_sequence(tcc:TCC) ->String{
    let mut seq = String::from("\x1b[");
    match tcc {
        TCC::Cpr => seq.push_str("6n"),
        TCC::CLeft(arg)=> seq.push_str(&format!("{}D", arg)),
        TCC::CDown(arg) => seq.push_str(&format!("{}B", arg)),
        TCC::CRight(arg)=> seq.push_str(&format!("{}C", arg)),
        TCC::CUp(arg)=> seq.push_str(&format!("{}A", arg)),
        TCC::PosC(row, column)=> seq.push_str(&format!("{};{}H", row, column)),
        TCC::EraseDisplay(erase)=> seq.push_str(&format!("{}J", resolve_erase(erase))),
        TCC::EraseLine(erase)=> seq.push_str(&format!("{}K", resolve_erase(erase))),
        TCC::Hide => seq.push_str("?25l"),
        TCC::Show => seq.push_str("?25h"),

    }
    seq
}

#[derive(Debug,PartialEq)]
pub enum ArrowDirection {
    Up,
    Down,
    Left,
    Right
}

#[derive(Debug,PartialEq)]
pub enum Key {
    Esc,
    Char(char),
    Ctrl(char),
    Arrow(ArrowDirection),
    NewLine,
    PageUp,
    PageDown,
    Home,
    End,
    Delete,
    Unknown
}
fn read_byte(fd : &mut impl Read)->Result<u8, io::Error>{
    let mut buf = [0;1];
    let mut n = 0;
    while n ==  0 {
        n = fd.read(&mut buf)?;
    }
    Ok(buf[0])
}

pub fn get_key_stroke(mut fd: &mut impl Read)->Result<Key, io::Error>{
    let b = read_byte(&mut fd);
    match b {
        Ok(b) => {
            if b == b'\x1b'{
                let sequence = [read_byte(&mut fd)?, read_byte(&mut fd)?];
                if sequence[0] == b'[' {
                    return match sequence[1] {
                        b'0'..=b'9' => {
                            let c = read_byte(&mut fd)?;
                            if c == b'~' {
                                return match sequence[1] {
                                    b'5' => Ok(Key::PageUp),
                                    b'6' => Ok(Key::PageDown),
                                    b'1'| b'7' | b'H' => Ok(Key::Home),
                                    b'4'| b'8' | b'F' => Ok(Key::End),
                                    b'3' => Ok(Key::Delete),
                                    _ => Ok(Key::Unknown)
                                }
                            } else {
                                Ok(Key::Unknown)
                            }
                        },
                        b'A' => Ok(Key::Arrow(ArrowDirection::Up)),
                        b'B' => Ok(Key::Arrow(ArrowDirection::Down)),
                        b'C' => Ok(Key::Arrow(ArrowDirection::Right)),
                        b'D' => Ok(Key::Arrow(ArrowDirection::Left)),
                        _ => Ok(Key::Unknown)
                    }
                }
                if sequence[0] == b'O' {
                    return match sequence[1] {
                        b'H' => Ok(Key::Home),
                        b'F' => Ok(Key::End),
                        _ => Ok(Key::Unknown)
                    }
                }
            }
            match b {
                27 => Ok(Key::Esc),

                10 | 13 => Ok(Key::NewLine),
                1..=26 => {
                    let c = (b + 64) as char;
                    Ok(Key::Ctrl(c))
                },
                _ => Ok(Key::Char(b as char))
            }
        },
        Err(_) => Ok(Key::Unknown)
    }
}
