use super::ast::*;
use super::binding_power::*;
use super::operators::*;
use super::tokeniser::{self, *};
use super::tokens::Token::*;
use super::tokens::*;
use std::iter::Peekable;

pub type Result<'a, X> = std::result::Result<X, ParseError<'a>>;

pub fn parse<'a>(source: &'a str) -> Result<'a, Ast<'a>> {
    Parser::of(tokenise(source)).parse()
}

#[derive(Debug, Clone)]
struct Parser<'a> {
    tokens: Peekable<Tokeniser<'a>>,
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
        let mut ast = Ast::new();

        while let Some(_) = self.peek_next_token()? {
            ast.append_statement(self.top_level_statement(false)?);
        }

        Ok(ast)
    }

    fn top_level_statement(&mut self, is_export: bool) -> Result<'a, TopLevelStatement<'a>> {
        use super::keywords::Keyword::*;

        if let Some(token) = self.step()? {
            match token {
                Name(name) => Ok(TopLevelStatement::Declaration {
                    decl: self.declaration(name)?,
                    exported: is_export,
                }),
                Keyword(Function) => Ok(TopLevelStatement::Declaration {
                    decl: self.function()?,
                    exported: is_export,
                }),
                Keyword(Export) => {
                    if is_export {
                        Err(ParseError::UnexpectedToken(token, "export specified twice"))
                    } else {
                        self.top_level_statement(true)
                    }
                }
                token => Err(ParseError::UnexpectedToken(token, "top level statement")),
            }
        } else {
            Err(ParseError::UnexpectedEndOfInput)
        }
    }

    fn func_body_statement(&mut self) -> Result<'a, FuncBodyStatement<'a>> {
        use super::keywords::Keyword::*;

        if let Some(token) = self.step()? {
            match (token, self.peek_next_token()?) {
                (Name(name), _) => self.named_statement(name),
                (Keyword(Function), _) => self.function().map(FuncBodyStatement::Declaration),
                (Constant(_), _) => Ok(FuncBodyStatement::BareExpression(
                    self.expression(0, Some(token))?,
                )),
                (OpenParen, _) => Ok(FuncBodyStatement::BareExpression(
                    self.expression(0, Some(token))?,
                )),
                (token, _) => Err(ParseError::UnexpectedToken(token, "function body")),
            }
        } else {
            Err(ParseError::UnexpectedEndOfInput)
        }
    }

    fn declaration(&mut self, name: &'a str) -> Result<'a, Declaration<'a>> {
        match self.step()? {
            Some(Equals) => Ok(Declaration::Assignment {
                name,
                expr: self.expression(0, None)?,
            }),
            Some(t) => Err(ParseError::UnexpectedToken(t, "top level assignment")),
            None => Err(ParseError::UnexpectedEndOfInput),
        }
    }

    fn named_statement(&mut self, name: &'a str) -> Result<'a, FuncBodyStatement<'a>> {
        use self::Declaration::*;
        use FuncBodyStatement::*;

        if let Some(Equals) = self.peek_next_token()? {
            self.step()?;

            Ok(Declaration(Assignment {
                name,
                expr: self.expression(0, None)?,
            }))
        } else {
            Ok(BareExpression(self.expression(0, Some(Token::Name(name)))?))
        }
    }

    fn function_call(&mut self, name: &'a str) -> Result<'a, Expression<'a>> {
        let mut args = vec![];

        while let Some(token) = self.step()? {
            match token {
                CloseParen => {
                    return Ok(Expression::FunctionCall { name, args });
                }
                _ => {
                    args.push(self.expression(0, Some(token))?);

                    if let Some(Comma) = self.peek_next_token()? {
                        self.step()?;
                    }
                }
            }
        }

        Err(ParseError::UnexpectedEndOfInput)
    }

    fn function(&mut self) -> Result<'a, Declaration<'a>> {
        match self.step()? {
            Some(Name(name)) => {
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
            Some(OpenParen) => {
                let mut args = Vec::new();

                while let Some(token) = self.step()? {
                    match token {
                        Name(name) => args.push(self.func_arg(name)?),
                        CloseParen => return Ok(FunctionArgsList { args }),
                        _ => return Err(ParseError::ErrorParsingFunctionArgs),
                    }
                }

                Err(ParseError::ErrorParsingFunctionArgs)
            }
            _ => Err(ParseError::ErrorParsingFunctionArgs),
        }
    }

    fn func_arg(&mut self, name: &'a str) -> Result<'a, FunctionArg<'a>> {
        if let Some(Comma) = self.peek_next_token()? {
            self.step()?;
        }

        Ok(FunctionArg { name })
    }

    fn function_body(&mut self) -> Result<'a, Vec<FuncBodyStatement<'a>>> {
        let mut statements = Vec::new();

        match self.step()? {
            Some(IndentIncr) => {}
            _ => return Err(ParseError::IndentExpectedError),
        }

        while let Some(token) = self.peek_next_token()? {
            match token {
                IndentDecr => {
                    self.step()?;
                    return Ok(statements);
                }
                _ => statements.push(self.func_body_statement()?),
            }
        }

        Err(ParseError::UnexpectedEndOfInput)
    }

    fn expression(
        &mut self,
        right_binding_power: u32,
        first_token: Option<Token<'a>>,
    ) -> Result<'a, Expression<'a>> {
        let mut current_token = match first_token {
            Some(t) => t,
            None => self
                .step()?
                .map_or(Err(ParseError::UnexpectedEndOfInput), Ok)?,
        };

        let mut left = self.null_denotation(current_token)?;

        while right_binding_power < self.next_token_binding_power() {
            current_token = self
                .step()?
                .map_or(Err(ParseError::UnexpectedEndOfInput), Ok)?;

            left = self.left_denotation(current_token, left)?;
        }

        Ok(left)
    }

    fn next_token_binding_power(&mut self) -> u32 {
        match self.peek_next_token() {
            Ok(Some(token)) => token.binding_power(),
            _ => 0,
        }
    }

    fn left_denotation(
        &mut self,
        token: Token<'a>,
        left: Expression<'a>,
    ) -> Result<'a, Expression<'a>> {
        match token {
            BinOp(operator) => {
                let right = self.expression(token.binding_power(), None)?;

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
            Constant(c) => Ok(Expression::Constant(c)),
            Name(name) => {
                if let Some(OpenParen) = self.peek_next_token()? {
                    self.step()?;
                    self.function_call(name)
                } else {
                    Ok(Expression::Variable(name))
                }
            }
            BinOp(BinaryOperator::Minus) => {
                Ok(Expression::Negation(Box::new(self.expression(100, None)?)))
            }
            OpenParen => {
                let expr = self.expression(token.binding_power(), None)?;

                match self.step()? {
                    Some(CloseParen) => Ok(expr),
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
