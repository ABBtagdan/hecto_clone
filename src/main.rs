#![warn(clippy::pedantic, clippy::all)]

mod document;
mod editor;
mod row;
mod terminal;
pub use document::Document;
pub use editor::Position;
pub use row::Row;
pub use terminal::Terminal;

use editor::Editor;

fn main() {
    Editor::default().run();
}
