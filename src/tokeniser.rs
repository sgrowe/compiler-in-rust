use super::keywords::*;
use super::tokens::*;
use std::str::CharIndices;

pub fn tokenise<'a>(source: &'a str) -> Tokeniser<'a> {
    let mut chars = source.char_indices();
    let next_char = chars.next();

    Tokeniser {
        source,
        chars,
        next_char,
        indent_stack: Vec::with_capacity(8),
    }
}

#[derive(Debug, Clone)]
pub struct Tokeniser<'a> {
    source: &'a str,
    chars: CharIndices<'a>,
    // the next char in the sequence so we can look ahead one char without consuming the iterator
    next_char: Option<(usize, char)>,
    indent_stack: Vec<u16>, // u16 as hopefully people wont use many more that 65,000 spaces of indentation
}

impl<'a> Iterator for Tokeniser<'a> {
    type Item = Result<Token<'a>, TokeniserError>;

    fn next(&mut self) -> Option<Self::Item> {
        use self::BinaryOperator::*;
        use Token::*;

        while let Some((i, c)) = self.step() {
            match c {
                ' ' => {
                    continue;
                }
                '\n' | '\r' => {
                    if let Some(token) = self.newline() {
                        return Some(Ok(token));
                    }

                    continue;
                }
                _ => {}
            }

            return match (c, self.peek_next_char()) {
                ('(', _) => Some(Ok(OpenParen)),
                (')', _) => Some(Ok(CloseParen)),
                ('"', _) => Some(self.string_constant(i)),
                ('|', _) => Some(Ok(Pipe)),
                ('+', _) => Some(Ok(BinOp(Plus))),
                ('*', _) => Some(Ok(BinOp(Multiply))),
                ('-', _) => Some(Ok(BinOp(Minus))),
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

        while let Some(_) = self.indent_stack.pop() {
            return Some(Ok(IndentDecr));
        }

        None
    }
}

impl<'a> Tokeniser<'a> {
    fn step(&mut self) -> Option<(usize, char)> {
        let current = self.next_char;
        self.next_char = self.chars.next();
        current
    }

    fn peek_next_char(&self) -> Option<char> {
        self.next_char.map(|(_, c)| c)
    }

    fn current_indent(&self) -> u16 {
        self.indent_stack.last().copied().unwrap_or(0)
    }

    fn newline(&mut self) -> Option<Token<'a>> {
        use Token::{IndentDecr, IndentIncr};

        let mut indent = 0;

        while let Some(c) = self.peek_next_char() {
            match c {
                ' ' => {
                    indent += 1;
                }
                '\n' | '\r' => {
                    // ignore blank lines
                    indent = 0;
                }
                _ => {
                    return if indent > self.current_indent() {
                        self.indent_stack.push(indent);
                        Some(IndentIncr)
                    } else if indent < self.current_indent() {
                        self.indent_stack.pop();
                        Some(IndentDecr)
                    } else {
                        None
                    };
                }
            }

            self.step();
        }

        None
    }

    fn string_constant(&mut self, start: usize) -> Result<Token<'a>, TokeniserError> {
        while let Some((i, c)) = self.step() {
            if c == '"' {
                return Ok(Token::Constant(Constant::Str(&self.source[start + 1..i])));
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

        get_matching_keyword(name)
            .map(|keyword| Token::Keyword(keyword))
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

        let constant = if is_float {
            Constant::Float(num.parse().unwrap())
        } else {
            Constant::Int(num.parse().unwrap())
        };

        Token::Constant(constant)
    }
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
    use test_case::test_case;

    #[test_case("src/fixtures/strings.lang"; "strings")]
    #[test_case("src/fixtures/maths.lang"; "maths")]
    #[test_case("src/fixtures/functions.lang"; "functions")]
    fn fixtures(fixture_file_name: &str) -> std::io::Result<()> {
        let contents = fs::read_to_string(fixture_file_name)?;

        let tokens = tokenise(&contents).collect::<Vec<_>>();

        assert_debug_snapshot!(tokens);

        Ok(())
    }
}
