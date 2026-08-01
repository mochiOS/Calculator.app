use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number(f64),
    Unary {
        operator: UnaryOperator,
        expression: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalculatorError {
    EmptyExpression,
    InvalidCharacter {
        character: char,
        position: usize,
    },
    InvalidNumber {
        position: usize,
    },
    UnexpectedToken {
        position: usize,
    },
    MissingRightParenthesis {
        position: usize,
    },
    DivisionByZero,
    NonFiniteResult,
}

impl fmt::Display for CalculatorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyExpression => {
                write!(formatter, "式が入力されていません")
            }
            Self::InvalidCharacter {
                character,
                position,
            } => {
                write!(
                    formatter,
                    "{}文字目の「{}」は使用できません",
                    position + 1,
                    character,
                )
            }
            Self::InvalidNumber {
                position,
            } => {
                write!(
                    formatter,
                    "{}文字目付近の数値が正しくありません",
                    position + 1,
                )
            }
            Self::UnexpectedToken {
                position,
            } => {
                write!(
                    formatter,
                    "{}文字目付近の式が正しくありません",
                    position + 1,
                )
            }
            Self::MissingRightParenthesis {
                position,
            } => {
                write!(
                    formatter,
                    "{}文字目付近に閉じ括弧が必要です",
                    position + 1,
                )
            }
            Self::DivisionByZero => {
                write!(formatter, "0では割れません")
            }
            Self::NonFiniteResult => {
                write!(formatter, "計算結果が大きすぎます")
            }
        }
    }
}

impl std::error::Error for CalculatorError {}

pub fn parse(source: &str) -> Result<Expression, CalculatorError> {
    let tokens = Lexer::new(source).tokenize()?;

    Parser::new(tokens).parse()
}

/// 計算式を入力すると解を返します。
///
/// ### Args
/// - `source: &str`: 計算式
///
/// ### Returns
/// 成功時は解（`f64`）を返し、失敗時は `CalculatorError` を返します
pub fn evaluate(source: &str) -> Result<f64, CalculatorError> {
    let expression = parse(source)?;
    let value = evaluate_expression(&expression)?;

    if !value.is_finite() {
        return Err(CalculatorError::NonFiniteResult);
    }

    Ok(value)
}

pub fn format_value(value: f64) -> String {
    if value == 0.0 {
        return String::from("0");
    }

    if value.fract().abs() < f64::EPSILON {
        return format!("{value:.0}");
    }

    let absolute = value.abs();

    if absolute >= 1_000_000_000_000.0 || absolute < 0.000_000_001 {
        return format!("{value:.10e}");
    }

    let text = format!("{value:.10}");

    text.trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParenthesis,
    RightParenthesis,
    End,
}

#[derive(Debug, Clone, PartialEq)]
struct Token {
    kind: TokenKind,
    position: usize,
}

struct Lexer<'a> {
    source: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, CalculatorError> {
        let mut tokens = Vec::new();

        while let Some(character) = self.current_character() {
            if character.is_whitespace() {
                self.advance_character();
                continue;
            }

            let position = self.position;

            let kind = match character {
                '0'..='9' | '.' => self.read_number()?,
                '+' => {
                    self.advance_character();
                    TokenKind::Plus
                }
                '-' | '−' => {
                    self.advance_character();
                    TokenKind::Minus
                }
                '*' | '×' => {
                    self.advance_character();
                    TokenKind::Star
                }
                '/' | '÷' => {
                    self.advance_character();
                    TokenKind::Slash
                }
                '(' => {
                    self.advance_character();
                    TokenKind::LeftParenthesis
                }
                ')' => {
                    self.advance_character();
                    TokenKind::RightParenthesis
                }
                _ => {
                    return Err(CalculatorError::InvalidCharacter {
                        character,
                        position,
                    });
                }
            };

            tokens.push(Token {
                kind,
                position,
            });
        }

        tokens.push(Token {
            kind: TokenKind::End,
            position: self.position,
        });

        Ok(tokens)
    }

    fn read_number(&mut self) -> Result<TokenKind, CalculatorError> {
        let start = self.position;
        let mut has_digit = false;
        let mut has_decimal_point = false;

        while let Some(character) = self.current_character() {
            match character {
                '0'..='9' => {
                    has_digit = true;
                    self.advance_character();
                }
                '.' if !has_decimal_point => {
                    has_decimal_point = true;
                    self.advance_character();
                }
                _ => break,
            }
        }

        if !has_digit {
            return Err(CalculatorError::InvalidNumber {
                position: start,
            });
        }

        if matches!(self.current_character(), Some('.')) {
            return Err(CalculatorError::InvalidNumber {
                position: self.position,
            });
        }

        let text = &self.source[start..self.position];
        let value = text
            .parse::<f64>()
            .map_err(|_| CalculatorError::InvalidNumber {
                position: start,
            })?;

        Ok(TokenKind::Number(value))
    }

    fn current_character(&self) -> Option<char> {
        self.source[self.position..].chars().next()
    }

    fn advance_character(&mut self) {
        if let Some(character) = self.current_character() {
            self.position += character.len_utf8();
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn parse(mut self) -> Result<Expression, CalculatorError> {
        if matches!(self.current().kind, TokenKind::End) {
            return Err(CalculatorError::EmptyExpression);
        }

        let expression = self.parse_addition()?;

        if !matches!(self.current().kind, TokenKind::End) {
            return Err(CalculatorError::UnexpectedToken {
                position: self.current().position,
            });
        }

        Ok(expression)
    }

    fn parse_addition(&mut self) -> Result<Expression, CalculatorError> {
        let mut expression = self.parse_multiplication()?;

        loop {
            let operator = match self.current().kind {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Subtract,
                _ => break,
            };

            self.advance();

            let right = self.parse_multiplication()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_multiplication(&mut self) -> Result<Expression, CalculatorError> {
        let mut expression = self.parse_unary()?;

        loop {
            let operator = match self.current().kind {
                TokenKind::Star => BinaryOperator::Multiply,
                TokenKind::Slash => BinaryOperator::Divide,
                _ => break,
            };

            self.advance();

            let right = self.parse_unary()?;

            expression = Expression::Binary {
                left: Box::new(expression),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expression)
    }

    fn parse_unary(&mut self) -> Result<Expression, CalculatorError> {
        let operator = match self.current().kind {
            TokenKind::Plus => Some(UnaryOperator::Positive),
            TokenKind::Minus => Some(UnaryOperator::Negative),
            _ => None,
        };

        if let Some(operator) = operator {
            self.advance();

            return Ok(Expression::Unary {
                operator,
                expression: Box::new(self.parse_unary()?),
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression, CalculatorError> {
        match self.current().kind.clone() {
            TokenKind::Number(value) => {
                self.advance();

                Ok(Expression::Number(value))
            }
            TokenKind::LeftParenthesis => {
                let opening_position = self.current().position;

                self.advance();

                let expression = self.parse_addition()?;

                if !matches!(self.current().kind, TokenKind::RightParenthesis) {
                    return Err(CalculatorError::MissingRightParenthesis {
                        position: opening_position,
                    });
                }

                self.advance();

                Ok(expression)
            }
            _ => Err(CalculatorError::UnexpectedToken {
                position: self.current().position,
            }),
        }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn advance(&mut self) {
        if self.position + 1 < self.tokens.len() {
            self.position += 1;
        }
    }
}

fn evaluate_expression(expression: &Expression) -> Result<f64, CalculatorError> {
    match expression {
        Expression::Number(value) => Ok(*value),
        Expression::Unary {
            operator,
            expression,
        } => {
            let value = evaluate_expression(expression)?;

            match operator {
                UnaryOperator::Positive => Ok(value),
                UnaryOperator::Negative => Ok(-value),
            }
        }
        Expression::Binary {
            left,
            operator,
            right,
        } => {
            let left = evaluate_expression(left)?;
            let right = evaluate_expression(right)?;

            match operator {
                BinaryOperator::Add => Ok(left + right),
                BinaryOperator::Subtract => Ok(left - right),
                BinaryOperator::Multiply => Ok(left * right),
                BinaryOperator::Divide => {
                    if right == 0.0 {
                        Err(CalculatorError::DivisionByZero)
                    } else {
                        Ok(left / right)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_result(source: &str, expected: f64) {
        let actual = evaluate(source).unwrap();

        assert!(
            (actual - expected).abs() < 0.000_000_001,
            "{source}: expected {expected}, got {actual}",
        );
    }

    #[test]
    fn evaluates_addition() {
        assert_result("1 + 2", 3.0);
    }

    #[test]
    fn respects_operator_precedence() {
        assert_result("1 + 2 * 3", 7.0);
    }

    #[test]
    fn evaluates_parentheses() {
        assert_result("(1 + 2) * 3", 9.0);
    }

    #[test]
    fn evaluates_unary_operators() {
        assert_result("-2 * -3", 6.0);
        assert_result("+4", 4.0);
    }

    #[test]
    fn accepts_calculator_symbols() {
        assert_result("10 ÷ 2 × 3 − 4", 11.0);
    }

    #[test]
    fn evaluates_decimal_numbers() {
        assert_result("0.5 + 1.25", 1.75);
    }

    #[test]
    fn rejects_division_by_zero() {
        assert_eq!(
			evaluate("10 / 0"),
			Err(CalculatorError::DivisionByZero),
		);
    }

    #[test]
    fn rejects_invalid_number() {
        assert!(matches!(
			evaluate("1..2"),
			Err(CalculatorError::InvalidNumber {
				..
			}),
		));
    }

    #[test]
    fn rejects_missing_parenthesis() {
        assert!(matches!(
			evaluate("(1 + 2"),
			Err(CalculatorError::MissingRightParenthesis {
				..
			}),
		));
    }

    #[test]
    fn formats_values() {
        assert_eq!(format_value(12.0), "12");
        assert_eq!(format_value(12.5), "12.5");
        assert_eq!(format_value(-0.0), "0");
    }
}
