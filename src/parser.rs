use super::ast::*;
use super::operators::*;
use super::tokeniser::*;
use super::tokens::Token::*;
use super::tokens::*;

pub fn parse<'a>(source: &'a str) -> Result<Ast<'a>, ParseError<'a>> {
    Parser::of(tokenise(source)).parse()
}

#[derive(Debug, Clone)]
struct Parser<'a> {
    next_token: Result<Option<Token<'a>>, ParseError<'a>>,
    tokens: Tokeniser<'a>,
}

fn next_token<'a>(tokens: &mut Tokeniser<'a>) -> Result<Option<Token<'a>>, ParseError<'a>> {
    match tokens.next() {
        Some(result) => result
            .map_err(ParseError::TokeniserError)
            .map(|token| Some(token)),
        None => Ok(None),
    }
}

impl<'a> Parser<'a> {
    fn of(mut tokens: Tokeniser<'a>) -> Parser<'a> {
        Parser {
            next_token: next_token(&mut tokens),
            tokens,
        }
    }

    fn step(&mut self) -> Result<Option<Token<'a>>, ParseError<'a>> {
        let current = self.next_token;
        self.next_token = next_token(&mut self.tokens);
        current
    }

    fn peek_next_token(&self) -> Result<Option<Token<'a>>, ParseError<'a>> {
        self.next_token
    }

    fn parse(&mut self) -> Result<Ast<'a>, ParseError<'a>> {
        let mut ast = Ast {
            statements: Vec::new(),
        };

        while let Some(_) = self.peek_next_token()? {
            ast.append_statement(self.statement()?);
        }

        Ok(ast)
    }

    fn statement(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        use super::keywords::Keyword::*;

        if let Some(token) = self.step()? {
            match (token, self.peek_next_token()?) {
                (Name(name), _) => self.named_statement(name),
                (Keyword(Function), _) => self.function(),
                (Constant(_), _) => Ok(Statement::BareExpression(self.expression(0, Some(token))?)),
                (token, _) => Err(ParseError::UnexpectedToken(token)),
            }
        } else {
            Err(ParseError::UnexpectedEndOfInput)
        }
    }

    fn named_statement(&mut self, name: &'a str) -> Result<Statement<'a>, ParseError<'a>> {
        use Statement::*;

        match self.peek_next_token()? {
            Some(Equals) => {
                self.step()?;

                Ok(Assignment {
                    name,
                    expr: self.expression(0, None)?,
                })
            }
            Some(IndentDecr) => Ok(BareExpression(Expression::Variable(name))),
            Some(_) => Ok(BareExpression(self.expression(0, Some(Token::Name(name)))?)),
            None => Ok(BareExpression(Expression::Variable(name))),
        }
    }

    fn function(&mut self) -> Result<Statement<'a>, ParseError<'a>> {
        match self.step()? {
            Some(Name(name)) => {
                let arguments = self.function_arguments_list()?;
                let body = self.function_body()?;

                Ok(Statement::FunctionDecl {
                    name,
                    arguments,
                    body,
                })
            }
            _ => Err(ParseError::FunctionParseError),
        }
    }

    fn function_arguments_list(&mut self) -> Result<FunctionArgsList<'a>, ParseError<'a>> {
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

    fn func_arg(&mut self, name: &'a str) -> Result<FunctionArg<'a>, ParseError<'a>> {
        match self.peek_next_token()? {
            Some(Comma) => {
                self.step()?;
            }
            _ => {}
        }

        Ok(FunctionArg { name })
    }

    fn function_body(&mut self) -> Result<Vec<Statement<'a>>, ParseError<'a>> {
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
                _ => statements.push(self.statement()?),
            }
        }

        Err(ParseError::UnexpectedEndOfInput)
    }

    fn expression(
        &mut self,
        right_binding_power: u32,
        first_token: Option<Token<'a>>,
    ) -> Result<Expression<'a>, ParseError<'a>> {
        let mut t = match first_token {
            Some(t) => t,
            None => self
                .step()?
                .map_or(Err(ParseError::UnexpectedEndOfInput), Ok)?,
        };

        let mut token = self.peek_next_token()?;

        let mut left = null_denotation(t);

        while right_binding_power < token.map(left_binding_power).unwrap_or(0) {
            t = token.map_or(Err(ParseError::UnexpectedEndOfInput), Ok)?;

            self.step()?;

            left = self.left_denotation(t, left)?;

            token = self.peek_next_token()?;
        }

        Ok(left)
    }

    fn left_denotation(
        &mut self,
        token: Token<'a>,
        left: Expression<'a>,
    ) -> Result<Expression<'a>, ParseError<'a>> {
        match token {
            BinOp(operator) => {
                let right = self.expression(left_binding_power(token), None)?;

                Ok(Expression::BinaryOp {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            _ => Ok(left),
        }
    }
}

fn null_denotation<'a>(token: Token<'a>) -> Expression<'a> {
    match token {
        Constant(c) => Expression::Constant(c),
        Name(name) => Expression::Variable(name),
        _ => panic!("unexpected token: {:?}", token),
    }
}

fn left_binding_power<'a>(token: Token<'a>) -> u32 {
    match token {
        BinOp(op) => op.binding_power(),
        _ => 0,
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ParseError<'a> {
    TokeniserError(TokeniserError),
    UnexpectedToken(Token<'a>),
    UnexpectedEndOfInput,
    FunctionParseError,
    ErrorParsingFunctionArgs,
    IndentExpectedError,
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

        let ast = parse(&contents);

        assert_debug_snapshot!(ast);

        Ok(())
    }
}
