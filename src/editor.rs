use std::fs::File;
use std::path::Path;
use std::io::{Read, stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color};

struct Document {
    lines: Vec<String>,
    len: usize,
}

#[derive(Debug)]
struct Coordinates {
    pub  x: usize,
    pub  y: usize,
}

pub struct Editor {
    document: Document,
    cursor: Coordinates,
    dirty: bool,
    terminal_size: Coordinates,
    path: String,
}

impl Editor {
    pub fn new(path : &Path) ->Editor {
        let mut document = Document {
            lines: Vec::new(),
            len: 0,
        };
        let cursor = Coordinates {
            x: 1,
            y: 0,
        };
        let dirty = false;
        let terminal_size = Coordinates {
            x: 0,
            y: 0,
        };
        let mut file  = File::open(path).unwrap();
        let mut filecontents = String::new();
        file.read_to_string(&mut filecontents).unwrap();
        let mut lines = filecontents.lines();
        while let Some(line) = lines.next() {
            document.lines.push(line.to_string());
            document.len += 1;
        }
        let mut editor = Editor {
            document,
            cursor,
            dirty,
            terminal_size,
            path: path.to_string_lossy().to_string(),
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
            let stdin = stdin();
            let key = stdin.keys().next();
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
                Key::Ctrl('s') => {
                    self.save();
                }
                _ => {}
            }
            self.render();
            stdout.flush().unwrap();
            self.dirty = true;
        }
    }

    fn add_line(&mut self, line: String) {
        self.document.lines.push(line);
        self.document.len += 1;
    }

    pub(crate) fn render(&mut self) {
        //maintain cursor position
        print!("{}{}",termion::clear::All, termion::cursor::Goto(1, 1));
        if self.document.len == 0 {
            self.add_line("\r".to_string());
        }

        //determine the start and end of the viewport
        let mut start = 0;
        let mut end = self.terminal_size.y;

        if self.document.len < self.terminal_size.y {
            end = self.document.len;
        }else if self.cursor.y > self.terminal_size.y {
            start = self.cursor.y.checked_sub(self.terminal_size.y).unwrap_or(0);
            end = self.cursor.y;
        }

        //print all lines
        for line in start..end {
            println!("{}\r",self.document.lines[line]);
        }
        println!("{}{} Cursor: {}:{}\t ViewPort: {}:{},\t {}{}",
                 color::Bg(color::Cyan),
                 color::Fg(color::Black),
                 self.cursor.x,
                 self.cursor.y,
                 start,end,
                 color::Fg(color::Reset),color::Bg(color::Reset));

        self.update_cursor_position(self.cursor.x, self.cursor.y);
    }

    fn insert_newline(&mut self) {
        //break the line at the cursor
        let mut line = self.document.lines[self.cursor.y].clone();
        let mut newline = line.split_off(self.cursor.x-1);
        if newline.len() == 0 {
            newline = "\r".to_string();
        }
        //remove the characters after the cursor
        self.document.lines[self.cursor.y].truncate(self.cursor.x-1);
        self.document.lines.insert(self.cursor.y, newline);
        self.cursor.y += 1;
        self.cursor.x = 0;
        self.document.len += 1;
    }
    fn insert_tab(&mut self) {
        self.insert_char('\t');
    }
    fn backspace(&mut self) {
        self.delete();
    }
    fn delete(&mut self) {
        let mut index = self.cursor.x.clone();
        if index > 0 {
            index -= 1;
        }
        let line = self.get_current_line();
        //check if there are any characters left in the line
        if line.chars().count() < 2 {
            //if there are no characters left, delete the line
            self.delete_line();
        }else{
            //find the character to delete
            line.remove(index);
            self.move_cursor_left();
        }
    }
    fn get_current_line(&mut self) -> &mut String {
        &mut self.document.lines[self.cursor.y]
    }

    fn insert_char(&mut self, c: char) {
        let index = self.cursor.x.clone();
        let line = self.get_current_line();
        line.insert(index, c);
        self.move_cursor_right();
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
        if self.cursor.y >= self.document.len {
            self.cursor.y = self.document.len.checked_sub(1).unwrap_or(0);
        }
        let line = &self.document.lines[self.cursor.y];
        if (self.cursor.x) >= line.chars().count() {
            self.cursor.x = line.chars().count();
        }
        if self.cursor.x < 1 {
            self.cursor.x = 1;
        }
        println!("{}", termion::cursor::Goto(self.cursor.x as u16, self.cursor.y as u16));
    }
    fn move_cursor_pageup(&mut self) {
        self.cursor.y = self.cursor.y.checked_sub(self.terminal_size.y).unwrap_or(0);
    }

    fn move_cursor_pagedown(&mut self) {
        self.cursor.y = self.cursor.y + self.terminal_size.y;
    }
    fn delete_line(&mut self) {
        self.document.lines.remove(self.cursor.y);
        self.document.len -= 1;
        self.cursor.y = self.cursor.y.checked_sub(1).unwrap_or(0);
    }
    fn save(&self) {
        if !self.dirty {
            return;
        }
        let mut file = File::create(self.path.clone()).unwrap();
        for line in &self.document.lines {
            file.write_all(line.as_bytes()).unwrap();
            file.write_all("\n".as_bytes()).unwrap();
        }

    }
}