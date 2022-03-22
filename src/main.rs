mod editor;

use std::env::args;
use std::path::Path;
use crate::editor::Editor;

struct Args {
    filename: String,
}



fn main() {
    match get_args() {
        Some(args) => {
            let path = Path::new(&args.filename);
            if path.exists() {
                let mut editor = Editor::new(path);
                editor.render();
                editor.start();
            } else {
                println!("File {} does not exist", &args.filename);
            }
        },
        None => {
            println!("Usage: <filename>");
        }
    }

}

fn get_args() -> Option<Args> {
    let mut args = args();
    args.next();
    args.next().map(|filename| Args {filename})
}
