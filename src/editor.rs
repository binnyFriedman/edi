use std::fs::File;
use std::io;
use std::path::Path;
use std::io::{Read, stdin, stdout, Write};
use std::ops::{Add, Range, Sub};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color};

struct Document {
    lines: Vec<String>,
    len: usize,
}

#[derive(Debug)]
struct Coordinates<T> {
      x: T,
      y: T,
      range: Range<T>,
}

impl<T: Add + Ord + Sub + Copy> Coordinates<T> {
    fn new(range:Range<T>) -> Coordinates<T> {
        Coordinates {
            x: range.start,
            y: range.start,
            range,
        }
    }
    fn set_x(&mut self, x: T) {
        if x < self.range.start {
            self.x = self.range.start;
        } else if x > self.range.end {
            self.x = self.range.end;
        } else {
            self.x = x;
        }
    }
    fn set_y(&mut self, y: T) {
        if y < self.range.start {
            self.y = self.range.start;
        } else if y > self.range.end {
            self.y = self.range.end;
        } else {
            self.y = y;
        }
    }
}

struct EditorCoords {
    cursor: Coordinates<usize>,
    term_size: Coordinates<usize>,
    range: Range<usize>,
}
enum EditorCoordinate{
    X,
    Y
}
impl EditorCoords{
    fn new(term_size:Coordinates<usize>) -> EditorCoords {
        let range = Range {
            start: 1,
            end: term_size.x,
        };
        EditorCoords {
            cursor: Coordinates::new(range.clone()),
            term_size,
            range,
        }
    }
   fn get_zero_based(&self,coord:EditorCoordinate) -> usize {
       match coord {
           EditorCoordinate::X => self.cursor.x - 1,
           EditorCoordinate::Y => self.cursor.y - 1,
       }
   }
    fn get_zero_x(&self) -> usize {
        self.get_zero_based(EditorCoordinate::X)
    }
    fn get_zero_y(&self) -> usize {
        self.get_zero_based(EditorCoordinate::Y)
    }
   fn get_x(&self) -> usize {
       self.cursor.x
   }
   fn get_y(&self) -> usize {
       self.cursor.y
   }
    fn up(&mut self) {
        self.cursor.set_y(self.cursor.y - 1);
    }
    fn down(&mut self,document_len:usize) {
        if self.cursor.y < document_len {
            self.cursor.set_y(self.cursor.y + 1);
        }
    }
    fn left(&mut self) {
        self.cursor.set_x(self.cursor.x - 1);
    }
    fn right(&mut self, max_x: usize) {
        if self.cursor.x < max_x {
            self.cursor.set_x(self.cursor.x + 1);
        }
    }
    fn home(&mut self) {
        self.cursor.set_x(self.range.start);
    }
    fn page_up(&mut self) {
        self.cursor.set_y(self.cursor.y - self.term_size.y);
    }
    fn page_down(&mut self) {
        self.cursor.set_y(self.cursor.y + self.term_size.y);
    }
}



pub struct Editor {
    document: Document,
    dirty: bool,
    coords: EditorCoords,
    path: String,
}

impl Editor {
    pub fn new(path : &Path) ->Editor {
        let mut document = Document {
            lines: Vec::new(),
            len: 0,
        };
        let(terminal_x, terminal_y) = termion::terminal_size().unwrap();
        let mut terminal_size = Coordinates::new(1..terminal_y as usize);
        terminal_size.set_x(terminal_x as usize);
        let dirty = false;
        let mut file  = File::open(path).unwrap();
        let mut filecontents = String::new();
        file.read_to_string(&mut filecontents).unwrap();
        let lines = filecontents.lines();
        for line in lines {
            document.lines.push(line.to_string());
            document.len += 1;
        }

        Editor {
            document,
            dirty,
            coords: EditorCoords::new(terminal_size),
            path: path.to_string_lossy().to_string(),
        }
    }


    pub fn start(&mut self){
        let mut stdout = stdout().into_raw_mode().unwrap();
        loop {
            let stdin = io::stdin();
            let stdin = stdin.lock();

            let key = stdin.keys().next().unwrap();

            let key = key.unwrap();
            match key {
                Key::Char('\n') => {
                    self.insert_newline();
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
                    self.coords.left();
                }
                Key::Right => {
                    self.coords.right(self.get_line_length());
                }
                Key::Up => {
                    self.coords.up();
                }
                Key::Down => {
                    self.coords.down(self.document.len);
                }
                Key::Home => {
                    self.coords.home();
                }
                Key::End => {
                    self.coords.cursor.set_x(self.get_line_length());
                }
                Key::PageUp => {
                    self.coords.page_up();
                }
                Key::PageDown => {
                    self.coords.page_down();
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

    fn get_line_length(&self) -> usize {
        self.document.lines[self.coords.get_zero_y()].chars().count()
    }

    pub(crate) fn render(&mut self) {
        //maintain cursor position
        print!("{}{}",termion::clear::All,termion::cursor::Goto(1, 1));
        if self.document.len == 0 {
            self.add_line("\r".to_string());
        }

        //determine the start and end of the viewport
        let mut start = 0;
        let mut end = self.coords.term_size.y;

        if self.document.len < self.coords.term_size.y {
            end = self.document.len;
        }else if self.coords.get_y() > self.coords.term_size.y {
            start = self.coords.get_zero_y() - self.coords.term_size.y;
            end = self.coords.get_y();
        }
        if end > self.document.len {
            end = self.document.len;
        }
        //print all lines
        for line in start..end {
            print!("{}\r\n",self.document.lines[line]);
        }
        self.print_footer(start, end);
    }

    fn print_footer(&mut self,start: usize, end: usize) {
        Editor::print_cursor(1, self.coords.term_size.y +1);
        println!("{}{} Cursor: {}:{}\t ViewPort: {}:{}\t,  TermSize: {}:{} \t {}{}",
                 color::Bg(color::Cyan),
                 color::Fg(color::Black),
                 self.coords.get_x(),
                 self.coords.get_y(),
                 start,end,
                 self.coords.term_size.x,
                 self.coords.term_size.y,
                 color::Fg(color::Reset),color::Bg(color::Reset));
        Editor::print_cursor(self.coords.get_x(), self.coords.get_y());
    }

    fn insert_newline(&mut self) {
        let y = self.coords.get_zero_based(EditorCoordinate::Y);
        let x = self.coords.get_zero_based(EditorCoordinate::X);
        //break the line at the cursor
        let mut line = self.document.lines[y].clone();
        let mut newline = line.split_off(x);
        if newline.chars().count() == 0 {
            newline = "\r".to_string();
        }
        //remove the characters after the cursor
        self.document.lines[y].truncate(x);
        self.document.lines.insert(y, newline);
        self.document.len += 1;
        self.coords.down(self.document.len);
        self.coords.cursor.set_x(1);
    }

    fn backspace(&mut self) {
        self.delete();
    }
    fn delete(&mut self) {
        let index = self.coords.get_zero_x();
        let line = self.get_current_line();
        //check if there are any characters left in the line
        if line.chars().count() < 2 {
            self.delete_line();
        }else{
            //find the character to delete
            line.remove(index);
            self.coords.left();
        }
    }
    fn get_current_line(&mut self) -> &mut String {
        &mut self.document.lines[self.coords.get_zero_y()]
    }

    fn insert_char(&mut self, c: char) {
        let index = self.coords.get_zero_x();
        let line = self.get_current_line();
        line.insert(index, c);
        self.coords.right(self.get_line_length());
    }
    fn print_cursor(x: usize, y: usize) {
        print!("{}", termion::cursor::Goto(x as u16, y as u16));
    }
    fn delete_line(&mut self) {
        self.document.lines.remove(self.coords.get_zero_based(EditorCoordinate::Y));
        self.document.len -= 1;
        self.coords.up();
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