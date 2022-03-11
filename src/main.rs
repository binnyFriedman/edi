mod editor;

use std::env::args;
use std::path::Path;
use crate::editor::Editor;

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
    let path = Path::new(&args.filename);
    if !path.exists() {
        println!("File does not exist");
        return;
    }
    //load editor with file
    let mut editor = Editor::new(path);
    editor.render();
    editor.start();
}

fn get_args() -> Option<Args> {
    let mut args = args();
    args.next();
    return match args.next() {
        Some(filename) => Some(Args {filename}),
        None => None
    };
}
