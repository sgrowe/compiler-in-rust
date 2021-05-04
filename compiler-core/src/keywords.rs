use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Keyword {
    Import,
    Export,
    Type,
    Function,
    If,
    Else,
}

pub fn get_matching_keyword(name: &str) -> Option<Keyword> {
    use Keyword::*;

    let keyword = match name {
        "import" => Import,
        "export" => Export,
        "type" => Type,
        "fn" => Function,
        "if" => If,
        "else" => Else,
        _ => return None,
    };

    Some(keyword)
}

impl From<Keyword> for &'static str {
    fn from(keyword: Keyword) -> Self {
        match keyword {
            Keyword::Import => "import",
            Keyword::Export => "export",
            Keyword::Type => "type",
            Keyword::Function => "fn",
            Keyword::If => "if",
            Keyword::Else => "else",
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
