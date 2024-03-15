use termion::color;

#[derive(PartialEq, Clone, Copy)]
pub enum Type {
    None,
    Number,
    Match,
    String,
    Char,
    Comment,
    PrimaryKeyword,
    SecondaryKeyword,
}

impl Type {
    pub fn to_color(self) -> impl color::Color {
        match self {
            Type::Comment => color::Rgb(133, 153, 0),
            Type::Number => color::Rgb(220, 163, 163),
            Type::Match => color::Rgb(38, 139, 210),
            Type::String => color::Rgb(211, 54, 130),
            Type::Char => color::Rgb(108, 113, 196),
            Type::PrimaryKeyword => color::Rgb(181, 137, 0),
            Type::SecondaryKeyword => color::Rgb(42, 161, 152),
            _ => color::Rgb(230, 230, 230),
        }
    }
}
