use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum Keyword {
    Import,
    Export,
    Type,
    Function,
}

pub fn get_matching_keyword(name: &str) -> Option<Keyword> {
    use Keyword::*;

    match name {
        "import" => Some(Import),
        "export" => Some(Export),
        "type" => Some(Type),
        "fn" => Some(Function),
        _ => None,
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Keyword::*;

        let as_str = match self {
            Import => "import",
            Export => "export",
            Type => "type",
            Function => "fn",
        };

        write!(f, "{}", as_str)
    }
}
