use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Keyword {
    Import,
    Export,
    Type,
    Function,
}

pub fn get_matching_keyword(name: &str) -> Option<Keyword> {
    use Keyword::*;

    let keyword = match name {
        "import" => Import,
        "export" => Export,
        "type" => Type,
        "fn" => Function,
        _ => return None,
    };

    Some(keyword)
}

impl Keyword {
    pub fn as_str(self) -> &'static str {
        use Keyword::*;

        match self {
            Import => "import",
            Export => "export",
            Type => "type",
            Function => "fn",
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
