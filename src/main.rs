mod editor;
mod termios_wrap;

use crate::editor::Editor;

fn main()->Result<(), std::io::Error> {
    let mut edi = Editor::new();
    // we pass the file name as the first argument
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        edi.open(&args[1])?;
    }
    edi.run()
}

