use crate::ast::*;
use crate::binding_power::*;
use crate::keywords::*;
use crate::operators::*;
use crate::tokeniser::{self, *};
use crate::tokens::*;
use std::iter::Peekable;

pub type Result<'a, X> = std::result::Result<X, ParseError<'a>>;

pub fn parse(source: &str) -> Result<Ast> {
    Parser::of(tokenise(source)).parse()
}

pub fn parse_iter(source: &str) -> Parser {
    Parser::of(tokenise(source))
}

#[derive(Debug)]
pub struct Parser<'a> {
    tokens: Peekable<Tokeniser<'a>>,
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<'a, TopLevelStatement<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.top_level_statement(false).transpose()
    }
}

impl<'a> Parser<'a> {
    fn of(tokens: Tokeniser<'a>) -> Parser<'a> {
        Parser {
            tokens: tokens.peekable(),
        }
    }

    fn step(&mut self) -> tokeniser::Result<Option<Token<'a>>> {
        self.tokens.next().transpose()
    }

    fn peek_next_token(&mut self) -> tokeniser::Result<Option<Token<'a>>> {
        self.tokens.peek().copied().transpose()
    }

    fn parse(&mut self) -> Result<'a, Ast<'a>> {
        let mut ast = Ast::default();

        loop {
            let statement = self.top_level_statement(false)?;

            match statement {
                Some(s) => ast.append_statement(s),
                None => return Ok(ast),
            }
        }
    }

    fn top_level_statement(
        &mut self,
        is_export: bool,
    ) -> Result<'a, Option<TopLevelStatement<'a>>> {
        if let Some(token) = self.step()? {
            match token {
                Token::Name(name) => Ok(Some(TopLevelStatement::Declaration {
                    decl: self.declaration(name)?,
                    exported: is_export,
                })),
                Token::Keyword(Keyword::Function) => Ok(Some(TopLevelStatement::Declaration {
                    decl: self.function()?,
                    exported: is_export,
                })),
                Token::Keyword(Keyword::Export) => {
                    if is_export {
                        Err(ParseError::UnexpectedToken(token, "export specified twice"))
                    } else {
                        self.top_level_statement(true)
                    }
                }
                Token::Keyword(Keyword::If) => todo!(),
                token => Err(ParseError::UnexpectedToken(token, "top level statement")),
            }
        } else {
            Ok(None)
        }
    }

    fn func_body_statement(&mut self) -> Result<'a, CodeBlockStatement<'a>> {
        if let Some(token) = self.step()? {
            match token {
                Token::Name(name) => self.named_statement(name),
                Token::Keyword(Keyword::Function) => {
                    self.function().map(CodeBlockStatement::Declaration)
                }
                Token::Keyword(Keyword::If) => self.if_statement(),
                Token::Constant(_) => Ok(CodeBlockStatement::BareExpression(
                    self.expression(None, Some(token))?,
                )),
                Token::OpenParen => Ok(CodeBlockStatement::BareExpression(
                    self.expression(None, Some(token))?,
                )),
                token => Err(ParseError::UnexpectedToken(token, "function body")),
            }
        } else {
            Err(ParseError::UnexpectedEndOfInput)
        }
    }

    fn declaration(&mut self, name: &'a str) -> Result<'a, Declaration<'a>> {
        match self.step()? {
            Some(Token::Equals) => Ok(Declaration::Assignment {
                name,
                expr: self.expression(None, None)?,
            }),
            Some(t) => Err(ParseError::UnexpectedToken(t, "top level assignment")),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    fn named_statement(&mut self, name: &'a str) -> Result<'a, CodeBlockStatement<'a>> {
        use self::Declaration::*;
        use CodeBlockStatement::*;

        if let Some(Token::Equals) = self.peek_next_token()? {
            self.step()?;

            Ok(Declaration(Assignment {
                name,
                expr: self.expression(None, None)?,
            }))
        } else {
            Ok(BareExpression(
                self.expression(None, Some(Token::Name(name)))?,
            ))
        }
    }

    fn function_call(&mut self, name: &'a str) -> Result<'a, Expression<'a>> {
        let mut args = vec![];

        while let Some(token) = self.step()? {
            match token {
                Token::CloseParen => {
                    return Ok(Expression::FunctionCall { name, args });
                }
                _ => {
                    args.push(self.expression(None, Some(token))?);

                    if let Some(Token::Comma) = self.peek_next_token()? {
                        self.step()?;
                    }
                }
            }
        }

        Err(ParseError::UnexpectedEndOfInput)
    }

    fn function(&mut self) -> Result<'a, Declaration<'a>> {
        match self.step()? {
            Some(Token::Name(name)) => {
                let arguments = self.function_arguments_list()?;
                let body = self.function_body()?;

                Ok(Declaration::FunctionDecl {
                    name,
                    arguments,
                    body,
                })
            }
            _ => Err(ParseError::FunctionParseError),
        }
    }

    fn function_arguments_list(&mut self) -> Result<'a, FunctionArgsList<'a>> {
        match self.step()? {
            Some(Token::OpenParen) => {
                let mut args = Vec::new();

                while let Some(token) = self.step()? {
                    match token {
                        Token::Name(name) => args.push(self.func_arg(name)?),
                        Token::CloseParen => return Ok(FunctionArgsList { args }),
                        _ => return Err(ParseError::ErrorParsingFunctionArgs),
                    }
                }

                Err(ParseError::ErrorParsingFunctionArgs)
            }
            _ => Err(ParseError::ErrorParsingFunctionArgs),
        }
    }

    fn func_arg(&mut self, name: &'a str) -> Result<'a, FunctionArg<'a>> {
        if let Some(Token::Comma) = self.peek_next_token()? {
            self.step()?;
        }

        Ok(FunctionArg { name })
    }

    fn function_body(&mut self) -> Result<'a, Vec<CodeBlockStatement<'a>>> {
        let mut statements = Vec::new();

        match self.step()? {
            Some(Token::IndentIncr) => {}
            _ => return Err(ParseError::IndentExpectedError),
        }

        while let Some(token) = self.peek_next_token()? {
            match token {
                Token::IndentDecr => {
                    self.step()?;
                    return Ok(statements);
                }
                _ => statements.push(self.func_body_statement()?),
            }
        }

        Err(ParseError::UnexpectedEndOfInput)
    }

    fn if_statement(&mut self) -> Result<'a, CodeBlockStatement<'a>> {
        // if keyword has already been consumed

        let condition = self.expression(None, None)?;

        let block = self.function_body()?;

        let mut cases = vec![IfStatementCase { condition, block }];

        let mut else_case = None;

        while let Some(Token::Keyword(Keyword::Else)) = self.peek_next_token()? {
            self.step()?;

            match self.peek_next_token()? {
                Some(Token::Keyword(Keyword::If)) => {
                    self.step()?;

                    let condition = self.expression(None, None)?;
                    let block = self.function_body()?;

                    cases.push(IfStatementCase { condition, block });
                }
                _ => {
                    let block = self.function_body()?;

                    else_case = Some(Box::new(block));

                    break;
                }
            }
        }

        Ok(CodeBlockStatement::IfStatement { cases, else_case })
    }

    fn expression(
        &mut self,
        right_binding_power: Option<BindingPower>,
        first_token: Option<Token<'a>>,
    ) -> Result<'a, Expression<'a>> {
        let mut current_token = match first_token {
            Some(t) => t,
            None => self
                .step()?
                .map_or(Err(ParseError::UnexpectedEndOfInput), Ok)?,
        };

        let mut left = self.null_denotation(current_token)?;

        while right_binding_power.unwrap_or_default() < self.next_token_binding_power() {
            current_token = self
                .step()?
                .map_or(Err(ParseError::UnexpectedEndOfInput), Ok)?;

            left = self.left_denotation(current_token, left)?;
        }

        Ok(left)
    }

    fn next_token_binding_power(&mut self) -> BindingPower {
        match self.peek_next_token() {
            Ok(Some(token)) => token.binding_power(),
            _ => BindingPower::default(),
        }
    }

    fn left_denotation(
        &mut self,
        token: Token<'a>,
        left: Expression<'a>,
    ) -> Result<'a, Expression<'a>> {
        match token {
            Token::BinOp(operator) => {
                let right = self.expression(Some(token.binding_power()), None)?;

                Ok(Expression::BinaryOp {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            _ => Ok(left),
        }
    }

    fn null_denotation(&mut self, token: Token<'a>) -> Result<'a, Expression<'a>> {
        match token {
            Token::Constant(c) => Ok(Expression::Constant(c)),
            Token::Name(name) => {
                if let Some(Token::OpenParen) = self.peek_next_token()? {
                    self.step()?;
                    self.function_call(name)
                } else {
                    Ok(Expression::Variable(name))
                }
            }
            Token::BinOp(BinaryOperator::Minus) => Ok(Expression::Negation(Box::new(
                self.expression(Some(BindingPower::negation()), None)?,
            ))),
            Token::OpenParen => {
                let expr = self.expression(Some(token.binding_power()), None)?;

                match self.step()? {
                    Some(Token::CloseParen) => Ok(expr),
                    Some(token) => Err(ParseError::UnexpectedToken(
                        token,
                        "parsing expression in brackets",
                    )),
                    None => Err(ParseError::UnexpectedEndOfInput),
                }
            }
            _ => Err(ParseError::UnexpectedToken(
                token,
                "null denotation of expression token",
            )),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ParseError<'a> {
    TokeniserError(TokeniserError),
    UnexpectedToken(Token<'a>, &'a str),
    UnexpectedEndOfInput,
    FunctionParseError,
    ErrorParsingFunctionArgs,
    IndentExpectedError,
    IfStatementBodyExpected,
}

impl<'a> From<TokeniserError> for ParseError<'a> {
    fn from(error: TokeniserError) -> Self {
        ParseError::TokeniserError(error)
    }
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
    #[test_case("src/fixtures/example_program.lang"; "example program")]
    fn fixtures(fixture_file_name: &str) -> std::io::Result<()> {
        let contents = fs::read_to_string(fixture_file_name)?;

        let ast = parse(&contents);

        assert_debug_snapshot!(ast);

        Ok(())
    }
}
