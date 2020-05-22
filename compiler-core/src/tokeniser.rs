use super::keywords::*;
use super::tokens::*;
use std::iter::Peekable;
use std::str::CharIndices;

use std::cmp::Ordering;

pub type Result<X> = std::result::Result<X, TokeniserError>;

pub fn tokenise<'a>(source: &'a str) -> Tokeniser<'a> {
    Tokeniser {
        source,
        chars: source.char_indices().peekable(),
        indent_stack: Vec::with_capacity(8),
    }
}

#[derive(Debug, Clone)]
pub struct Tokeniser<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
    indent_stack: Vec<u16>, // u16 as hopefully people wont use many more that 65,000 spaces of indentation
}

impl<'a> Iterator for Tokeniser<'a> {
    type Item = Result<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        use super::operators::BinaryOperator::*;
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

            let token = match c {
                '(' => OpenParen,
                ')' => CloseParen,
                '"' => match self.string_constant(i) {
                    Ok(str_const) => str_const,
                    Err(error) => return Some(Err(error)),
                },
                '|' => Pipe,
                '+' => BinOp(Plus),
                '*' => BinOp(Multiply),
                '-' => BinOp(Minus),
                '/' => BinOp(Divide),
                ',' => Comma,
                '=' => match self.peek_next_char() {
                    Some('>') => {
                        self.step();
                        FatRightArrow
                    }
                    _ => Equals,
                },
                _ => {
                    if c.is_alphabetic() {
                        self.name(i)
                    } else if c.is_numeric() {
                        self.number(i)
                    } else {
                        return None;
                    }
                }
            };

            return Some(Ok(token));
        }

        if self.indent_stack.pop().is_some() {
            Some(Ok(IndentDecr))
        } else {
            None
        }
    }
}

impl<'a> Tokeniser<'a> {
    fn step(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }

    fn peek_next_char(&mut self) -> Option<char> {
        self.chars.peek().map(|&(_, c)| c)
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
                    return match indent.cmp(&self.current_indent()) {
                        Ordering::Greater => {
                            self.indent_stack.push(indent);
                            Some(IndentIncr)
                        }
                        Ordering::Less => {
                            self.indent_stack.pop();
                            Some(IndentDecr)
                        }
                        Ordering::Equal => None,
                    };
                }
            }

            self.step();
        }

        None
    }

    fn string_constant(&mut self, start: usize) -> Result<Token<'a>> {
        while let Some((i, c)) = self.step() {
            if c == '"' {
                return Ok(Token::Constant(Constant::Str(&self.source[start + 1..i])));
            }
        }

        Err(TokeniserError::UnterminatedString)
    }

    fn name(&mut self, start: usize) -> Token<'a> {
        let mut end = start + 1;

        while let Some(&(i, c)) = self.chars.peek() {
            end = i;

            if c.is_alphabetic() || c == '_' {
                self.step();
            } else {
                break;
            }
        }

        let name = &self.source[start..end];

        get_matching_keyword(name)
            .map(Token::Keyword)
            .unwrap_or(Token::Name(name))
    }

    fn number(&mut self, start: usize) -> Token<'a> {
        let mut end = start + 1;
        let mut is_float = false;

        while let Some(&(i, c)) = self.chars.peek() {
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
