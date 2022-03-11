
use std::env::args;
use std::fs;
use std::fs::File;
use std::io::{Read, stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color,style};

struct Document {
    lines: Vec<String>,
    len: usize,
}

#[derive(Debug)]
struct Coordinates {
 pub  x: usize,
 pub  y: usize,
}

struct Editor {
    document: Document,
    cursor: Coordinates,
    dirty: bool,
    terminal_size: Coordinates,
    filename: String,
}

impl Editor {
    pub fn new(mut file:File)->Editor {
        let mut document = Document {
            lines: Vec::new(),
            len: 0,
        };
        let cursor = Coordinates {
            x: 1,
            y: 1,
        };
        let dirty = false;
        let terminal_size = Coordinates {
            x: 1,
            y: 1,
        };
        let mut filecontents = String::new();
        file.read_to_string(&mut filecontents).unwrap();
        let mut lines = filecontents.lines();
        while let Some(line) = lines.next() {
            document.lines.push(line.to_string());
        }
        document.len = document.lines.len();
        let mut editor = Editor {
            document,
            cursor,
            dirty,
            terminal_size,
            filename: filecontents,
        };
        editor.update_terminal_size();
        editor
    }

    pub fn update_terminal_size(&mut self) {
        let (x,y) = termion::terminal_size().unwrap();
        self.terminal_size.x = x as usize;
        self.terminal_size.y = y as usize;
    }

    pub fn start(&mut self){
        let mut stdout = stdout().into_raw_mode().unwrap();
        loop {
            let mut stdin = stdin();
            let mut key = stdin.keys().next();
            if key.is_none() {
                continue;
            }
            let key = key.unwrap().unwrap();
            match key {
                Key::Char('\n') => {
                    self.insert_newline();
                }
                Key::Char('\t') => {
                    self.insert_tab();
                }
                Key::Char(c) => {
                    self.insert_char(c);
                }
                Key::Backspace => {
                    self.backspace();
                }
                Key::Delete => {
                    self.delete();
                }
                Key::Left => {
                    self.move_cursor_left();
                }
                Key::Right => {
                    self.move_cursor_right();
                }
                Key::Up => {
                    self.move_cursor_up();
                }
                Key::Down => {
                    self.move_cursor_down();
                }
                Key::Home => {
                    self.move_cursor_home();
                }
                Key::End => {
                    self.move_cursor_end();
                }
                Key::PageUp => {
                    self.move_cursor_pageup();
                }
                Key::PageDown => {
                    self.move_cursor_pagedown();
                }
                Key::Ctrl('q') => {
                    break;
                }
                _ => {}
            }
            self.render();
            stdout.flush().unwrap();
            self.dirty = true;
        }
    }

    fn render(&mut self) {
        //maintain cursor position
        print!("{}{}",termion::clear::All, termion::cursor::Goto(1, 0));

        //determine the start and end of the viewport
        let mut start = 0;
        let mut end = self.terminal_size.y;
        if self.cursor.y > end {
            start = self.cursor.y - end + 1;
            end = self.cursor.y;
        }
        if end > self.document.len {
            end = self.document.len;
        }

        //print all lines
       for line in start..end {
           println!("{}\r", self.document.lines[line]);
       }
       //print cursor
       //  println!("{}{} Cursor: {}:{}\r{}{}",color::Bg(color::Cyan),color::Fg(color::Black),self.cursor.x,self.cursor.y,color::Fg(color::Reset),color::Bg(color::Reset));
        self.update_cursor_position(self.cursor.x, self.cursor.y);
    }

    fn insert_newline(&self) {
        todo!()
    }
    fn insert_tab(&self) {
        todo!()
    }
    fn backspace(&self) {
        todo!()
    }
    fn delete(&self) {
        todo!()
    }
    fn insert_char(&self, c: char) {
        todo!()
    }
    fn move_cursor_left(&mut self) {
        self.cursor.x -= 1;
    }
    fn move_cursor_right(&mut self) {
        self.cursor.x += 1;
    }
    fn move_cursor_up(&mut self) {
        if self.cursor.y > 0 {
            self.cursor.y -= 1;
        }
    }

    fn move_cursor_down(&mut self) {
        self.cursor.y += 1;
    }
    fn move_cursor_home(&mut self) {
        self.cursor.x = 1;
    }
    fn move_cursor_end(&mut self) {
        self.cursor.x = self.terminal_size.x;
    }
    fn update_cursor_position(&mut self,x: usize, y: usize) {
        self.cursor.x = x;
        self.cursor.y = y;
        if self.cursor.x > self.terminal_size.x {
            self.cursor.x = self.terminal_size.x;
        }
        if self.cursor.y > self.document.len {
            self.cursor.y = self.document.len;
        }
        if self.cursor.x < 1 {
            self.cursor.x = 1;
        }
        if self.cursor.y < 0 {
            self.cursor.y = 0;
        }
        println!("{}", termion::cursor::Goto(self.cursor.x as u16, self.cursor.y as u16));
    }
    fn move_cursor_pageup(&mut self) {
      self.cursor.y = self.cursor.y - self.terminal_size.y;
    }

    fn move_cursor_pagedown(&mut self) {
      self.cursor.y = self.cursor.y + self.terminal_size.y;
    }
}

struct Args {
    filename: String,
}

fn main() {
    let args = get_args();
    if args.is_none() {
        println!("Usage: <filename>");
        return;
    }
    let args = args.unwrap();
    //try to open file
    let mut file = match fs::File::open(&args.filename) {
        Ok(file) => file,
        Err(_) => {
            println!("File {} not found", args.filename);
            return;
        }
    };
    //load editor with file
    let mut editor = Editor::new(file);
    editor.render();
    //start editor
    editor.start();
    //if the editor returns, we exit
    return;
}

fn get_args() -> Option<Args> {
    let mut args = args();
    args.next();
    return match args.next() {
        Some(filename) => Some(Args {filename}),
        None => None
    };
}
