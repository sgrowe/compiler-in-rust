use crate::keywords::*;
use crate::tokens::*;
use std::cmp::Ordering;
use std::iter::Peekable;
use std::str::CharIndices;
use tinyvec::TinyVec;

pub type Result<X> = std::result::Result<X, TokeniserError>;

pub fn tokenise(source: &str) -> Tokeniser {
    Tokeniser {
        source,
        chars: source.char_indices().peekable(),
        indent_stack: TinyVec::new(),
        current_line_indent: 0,
    }
}

type Indent = u16; // u16 as hopefully people wont use many more than 65,000 spaces of indentation

#[derive(Debug)]
pub struct Tokeniser<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
    indent_stack: TinyVec<[Indent; 14]>,
    current_line_indent: Indent,
}

impl<'a> Iterator for Tokeniser<'a> {
    type Item = Result<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        use super::operators::BinaryOperator::*;
        use Token::*;

        if self.indent_level() > self.current_line_indent {
            self.indent_stack.pop();

            return Some(Ok(IndentDecr));
        }

        while let Some((i, c)) = self.step() {
            let token = match c {
                // handle whitespace
                ' ' => continue,
                '\n' | '\r' => match self.newline() {
                    Some(token) => token,
                    None => continue,
                },

                // actual tokens
                '(' => OpenParen,
                ')' => CloseParen,
                '"' => match self.string_constant(i) {
                    Ok(str_const) => str_const,
                    Err(error) => return Some(Err(error)),
                },
                ':' => Colon,
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
                    Some('=') => {
                        self.step();
                        BinOp(DoubleEquals)
                    }
                    _ => Equals,
                },
                _ => {
                    if c.is_alphabetic() {
                        self.name(i)
                    } else if c.is_numeric() {
                        self.number(i)
                    } else {
                        todo!("Unexpected char: {}", c)
                    }
                }
            };

            return Some(Ok(token));
        }

        self.indent_stack.pop().map(|_| Ok(IndentDecr))
    }
}

impl<'a> Tokeniser<'a> {
    fn step(&mut self) -> Option<(usize, char)> {
        self.chars.next()
    }

    fn peek_next_char(&mut self) -> Option<char> {
        self.chars.peek().map(|&(_, c)| c)
    }

    fn indent_level(&self) -> Indent {
        self.indent_stack.last().copied().unwrap_or_default()
    }

    // outputs tokens on indentation change
    fn newline(&mut self) -> Option<Token<'a>> {
        use Token::{IndentDecr, IndentIncr};

        self.current_line_indent = 0;

        while let Some(c) = self.peek_next_char() {
            match c {
                ' ' => {
                    self.current_line_indent += 1;
                }
                '\n' | '\r' => {
                    // ignore blank lines
                    self.current_line_indent = 0;
                }
                _ => {
                    return match self.current_line_indent.cmp(&self.indent_level()) {
                        Ordering::Greater => {
                            self.indent_stack.push(self.current_line_indent);
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

    // TODO: single quoted strings
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

    #[test_case("strings")]
    #[test_case("maths")]
    #[test_case("functions")]
    #[test_case("fibonacci")]
    fn fixtures(name: &str) {
        let contents = fs::read_to_string(&format!("src/fixtures/{}.lang", name)).unwrap();

        let tokens = tokenise(&contents).collect::<Vec<_>>();

        assert_debug_snapshot!(tokens);
    }
}
