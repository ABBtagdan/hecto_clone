#![warn(clippy::pedantic, clippy::all)]

mod document;
mod editor;
mod filetype;
mod highlight;
mod row;
mod terminal;

pub use document::Document;
pub use editor::Position;
pub use editor::SearchDirection;
pub use filetype::FileType;
pub use filetype::HighlightingOptions;
pub use row::Row;
pub use terminal::Terminal;

use editor::Editor;

fn main() {
    Editor::default().run();
}
