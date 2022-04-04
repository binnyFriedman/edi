mod editor;
mod termios_wrap;

use crate::editor::Editor;

fn main()->Result<(), std::io::Error> {
    let mut edi = Editor::new();
    edi.run()
}

