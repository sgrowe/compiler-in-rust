use super::ast::*;
use super::tokeniser::*;
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
        use super::tokens::Token::*;

        let mut ast = Ast {
            statements: Vec::new(),
        };

        while let Some(token) = self.step()? {
            match token {
                Name(name) => ast.statements.push(self.statement(name)?),
                _ => break,
            }
        }

        Ok(ast)
    }

    fn statement(&mut self, name: &'a str) -> Result<TopLevelStatement<'a>, ParseError<'a>> {
        use TopLevelStatement::*;

        match self.step()? {
            Some(Token::Equals) => Ok(Assignment {
                name,
                expr: self.expression(0)?,
            }),
            Some(token) => Err(ParseError::UnexpectedToken(token)),
            None => Ok(BareExpression(Expression::Variable(name))),
        }
    }

    fn expression(&mut self, right_binding_power: u32) -> Result<Expression<'a>, ParseError<'a>> {
        let mut t = self
            .step()?
            .map_or(Err(ParseError::UnexpectedEndOfInput), |t| Ok(t))?;

        let mut token = self.peek_next_token()?;

        println!("Two Ts: {:?} {:?}", t, token);

        // let mut token = match self.next_token()? {
        //     Some(Constant(c)) => Expression::Constant(c),
        //     Some(Name(name)) => Expression::Variable(name),
        //     Some(token) => return Err(ParseError::UnexpectedToken(token)),
        //     None => return Err(ParseError::UnexpectedEndOfInput),
        // };

        let mut left = null_denotation(t);

        while right_binding_power < token.map(left_binding_power).unwrap_or(0) {
            t = token.map_or(Err(ParseError::UnexpectedEndOfInput), |t| Ok(t))?;

            println!("T: {:?}", t);

            self.step()?;

            token = self.peek_next_token()?;

            // self
            //     .step()?
            //     .map_or(Err(ParseError::UnexpectedEndOfInput), |t| Ok(t))?;

            println!("token: {:?}", token);

            left = self.left_denotation(t, left)?;
        }

        Ok(left)
    }

    fn left_denotation(
        &mut self,
        token: Token<'a>,
        left: Expression<'a>,
    ) -> Result<Expression<'a>, ParseError<'a>> {
        println!("Left denot: {:?}", token);

        match token {
            Token::BinOp(operator) => {
                let right = self.expression(left_binding_power(token))?;

                println!("Right: {:?}", right);

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
    println!("null d: {:?}", token);

    match token {
        Token::Constant(c) => Expression::Constant(c),
        Token::Name(name) => Expression::Variable(name),
        _ => panic!("bad"),
    }
}

fn left_binding_power<'a>(token: Token<'a>) -> u32 {
    match token {
        Token::BinOp(BinaryOperator::Plus) => 10,
        Token::BinOp(BinaryOperator::Minus) => 10,
        _ => 0,
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ParseError<'a> {
    TokeniserError(TokeniserError),
    UnexpectedToken(Token<'a>),
    UnexpectedEndOfInput,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
    use std::fs;
    use test_case::test_case;

    #[test_case("src/fixtures/strings.lang")]
    #[test_case("src/fixtures/maths.lang")]
    fn fixture_tests(fixture_file_name: &str) -> std::io::Result<()> {
        let contents = fs::read_to_string(fixture_file_name)?;

        let ast = parse(&contents);

        assert_debug_snapshot!(ast);

        Ok(())
    }
}
