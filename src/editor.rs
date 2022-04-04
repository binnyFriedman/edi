use std::io;
use std::io::{stdout, Write};
use crate::termios_wrap;
use crate::termios_wrap::{ArrowDirection, Erase, get_key_stroke, get_window_size, Key, TCC, term_sequence};


struct Buffer {
    data: Vec<char>,
    cursor: usize
}

impl Buffer{
    fn new()->Buffer{
        Buffer{
            data: Vec::new(),
            cursor: 0
        }
    }
    fn append_str(&mut self, s: &str){
        for c in s.chars(){
            self.data.push(c);
        }
    }
    fn append_line(&mut self, s: &str){
        self.append_str(s);
        self.data.push('\r');
        self.data.push('\n');
    }
    fn append_char(&mut self, c: char){
        self.data.push(c);
    }
    fn get_all(&self)->String{
        self.data.iter().collect()
    }
    fn clear(&mut self){
        self.data.clear();
        self.cursor = 0;
    }
}


pub struct Editor {
    screen_rows: usize,
    screen_cols: usize,
    cx: usize,
    cy: usize,
    buffer: Buffer,
}
impl Editor{
    pub(crate) fn new() ->Editor{
        Editor{
            screen_rows: 0,
            screen_cols: 0,
            cx: 0,
            cy: 0,
            buffer: Buffer::new()
        }
    }
    fn update_screen_size(&mut self){
        let (rows, cols) = get_window_size();
        self.screen_rows = rows as usize;
        self.screen_cols = cols as usize;
    }

    pub(crate) fn run(&mut self) ->Result<(),io::Error>{
        let _termios = termios_wrap::TermiosWrap::new();
        self.update_screen_size();
        let mut stdin = io::stdin();
        loop {
            self.refresh_screen()?;
            self.print_buffer()?;
            let key = get_key_stroke(&mut stdin)?;
            self.process_key(key)?;
        }
    }
    fn header(&mut self)->String{
        let message = "Welcome to Edi";
        let padding = (self.screen_cols - message.len() - 2)/2;
        let padding_str = "=".repeat(padding);
        format!("{} {} {}",padding_str,message,padding_str)
    }
    fn refresh_screen(&mut self)->Result<(), io::Error>{
        self.buffer.append_str(&term_sequence(TCC::Hide));
        self.buffer.append_str(&term_sequence(TCC::PosC(1, 1)));
        self.draw_rows();
        self.append_cursor();
        self.buffer.append_str(&term_sequence(TCC::Show ));
        Ok(())
    }

    fn draw_rows(&mut self){
        for index in 0..self.screen_rows{
            if index == self.screen_rows / 3 {
                let message = self.header();
                self.buffer.append_str(&message);
            }else{
                self.buffer.append_char('~');
            }
            self.buffer.append_line(&term_sequence(TCC::EraseLine(Erase::FromCursor)));
        }
    }
    fn process_arrow(&mut self,dir:ArrowDirection){
        match dir {
            ArrowDirection::Up => {
                if self.cy > 0 {
                    self.cy -= 1;
                }
            },
            ArrowDirection::Down => {
                if self.cy < self.screen_rows - 1 {
                    self.cy += 1;
                }
            },
            ArrowDirection::Right => {
                if self.cx < self.screen_cols - 1 {
                    self.cx += 1;
                }
            },
            ArrowDirection::Left => {
                if self.cx > 0 {
                    self.cx -= 1;
                }
            }
        }
    }
    fn process_key(&mut self, key: Key) ->Result<(), io::Error>{
        match key {
            Key::Esc => {
                self.cx = 0;
                self.cy = 0;
            },
            Key::Char(c) => {
                self.buffer.append_char(c);
            },
            Key::Arrow(dir)=>{
                self.process_arrow(dir);
            }
            Key::Ctrl('Q') => {
                self.clear_screen()?;
                std::process::exit(0);
            },
            Key::PageUp | Key::PageDown =>{
                for _ in  1..self.screen_rows{
                    self.process_arrow(if key == Key::PageUp {ArrowDirection::Up} else {ArrowDirection::Down});
                }
            },
            Key::Home =>{
                self.cx = 0;
            },
            Key::End =>{
                self.cx = self.screen_cols - 1;
            },
            Key::Delete => {
                self.process_arrow(ArrowDirection::Left);
            }
            Key::NewLine => {
                self.buffer.append_line("");
            },
            Key::Unknown => {
                println!("Unknown key {:?}", key);
            }
            _ => {
                println!("{:?}", key);
            }
        }
        Ok(())
    }
    fn clear_screen(&self)->Result<(), io::Error>{
        print!("{}", term_sequence(TCC::EraseDisplay(Erase::All)));
        print!("{}", term_sequence(TCC::PosC(1, 1), ));
        stdout().flush()?;
        Ok(())
    }

    fn print_buffer(&mut self) -> Result<(), io::Error>{
        let buffer_str = self.buffer.get_all();
        print!("{}", buffer_str);
        stdout().flush()?;
        self.buffer.clear();
        Ok(())
    }

    fn append_cursor(&mut self){
        self.buffer.append_str(&term_sequence(TCC::PosC(self.cy+1,self.cx+1)));
    }
}
