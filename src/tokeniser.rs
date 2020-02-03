use std::str::CharIndices;

pub fn tokenise<'a>(source: &'a str) -> Tokeniser<'a> {
    let mut chars = source.char_indices();
    let next_char = chars.next();

    Tokeniser {
        source,
        next_char,
        chars,
    }
}

pub struct Tokeniser<'a> {
    source: &'a str,
    chars: CharIndices<'a>,
    // the next char in the sequence so we can lookahead one char without consuming the iterator
    next_char: Option<(usize, char)>,
}

impl<'a> Iterator for Tokeniser<'a> {
    type Item = Result<Token<'a>, TokeniserError>;

    fn next(&mut self) -> Option<Self::Item> {
        use Token::*;
        use TokeniserError::*;

        while let Some((i, c)) = self.step() {
            if c.is_whitespace() {
                continue;
            }

            return match (c, self.peek_next_char()) {
                ('"', _) => Some(self.string_constant(i)),
                ('|', _) => Some(Ok(Pipe)),
                ('=', Some('>')) => {
                    self.step();
                    Some(Ok(RightArrow))
                }
                ('=', _) => Some(Ok(Equals)),
                (_, _) => {
                    if c.is_alphabetic() {
                        Some(Ok(self.name(i)))
                    } else if c.is_numeric() {
                        Some(Ok(self.number(i)))
                    } else {
                        None
                    }
                }
            };
        }

        None
    }
}

const KEYWORDS: [(&str, Keyword); 4] = [
    ("import", Keyword::Import),
    ("export", Keyword::Export),
    ("type", Keyword::Type),
    ("fn", Keyword::Function),
];

impl<'a> Tokeniser<'a> {
    fn step(&mut self) -> Option<(usize, char)> {
        let current = self.next_char;
        self.next_char = self.chars.next();
        current
    }

    fn peek_next_char(&self) -> Option<char> {
        self.next_char.map(|(_, c)| c)
    }

    fn string_constant(&mut self, start: usize) -> Result<Token<'a>, TokeniserError> {
        while let Some((i, c)) = self.step() {
            if c == '"' {
                return Ok(Token::StringConstant(&self.source[start + 1..i]));
            }
        }

        Err(TokeniserError::UnterminatedString)
    }

    fn name(&mut self, start: usize) -> Token<'a> {
        let mut end = start + 1;

        while let Some((i, c)) = self.next_char {
            end = i;

            if c.is_alphabetic() || c == '_' {
                self.step();
            } else {
                break;
            }
        }

        let name = &self.source[start..end];

        KEYWORDS
            .iter()
            .find(|&(string, _)| string == &name)
            .map(|&(_, keyword)| Token::Keyword(keyword))
            .unwrap_or(Token::Name(name))
    }

    fn number(&mut self, start: usize) -> Token<'a> {
        let mut end = start + 1;
        let mut is_float = false;

        while let Some((i, c)) = self.next_char {
            end = i;

            if c.is_numeric() {
                self.step();
            } else if !is_float && c == '.' {
                is_float = true;
                self.step();
            } else {
                break;
            }
        }

        let num = &self.source[start..end];

        if is_float {
            Token::FloatConstant(num.parse().unwrap())
        } else {
            Token::IntConstant(num.parse().unwrap())
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Token<'a> {
    RightArrow,
    Equals,
    Pipe,
    Name(&'a str),
    Keyword(Keyword),
    StringConstant(&'a str),
    FloatConstant(f64),
    IntConstant(i64),
}

#[derive(Debug, Copy, Clone)]
pub enum Keyword {
    Import,
    Export,
    Type,
    Function,
}

#[derive(Debug, Copy, Clone)]
pub enum TokeniserError {
    UnterminatedString,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
    use std::fs;

    #[test]
    fn test_add() -> std::io::Result<()> {
        let contents = fs::read_to_string("src/fixtures/strings.lang")?;

        let tokens = tokenise(&contents).collect::<Vec<_>>();

        assert_debug_snapshot!(tokens);

        Ok(())
    }
}
