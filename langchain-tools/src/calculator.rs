//! Calculator tool for math expressions.

use async_trait::async_trait;
use langchain_core::errors::ChainError;

use super::traits::{BaseTool, ToolResult};

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LeftParen,
    RightParen,
    UnaryMinus,
    Function(String),
}

pub struct CalculatorTool;

#[async_trait]
impl BaseTool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Useful for performing mathematical calculations. Input should be a mathematical expression. Supports +, -, *, /, ^, sqrt, sin, cos, tan, log, ln, abs, floor, ceil, round, pi, e, and parentheses."
    }

    async fn invoke(&self, input: &str) -> ToolResult {
        let input = input.trim();
        if input.is_empty() {
            return Err(ChainError::ToolError("Empty expression".into()));
        }
        let tokens = tokenize(input)?;
        let rpn = shunting_yard(tokens)?;
        let result = evaluate_rpn(rpn)?;
        Ok(result.to_string())
    }
}

fn tokenize(input: &str) -> std::result::Result<Vec<Token>, ChainError> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }

        if chars[i].is_ascii_digit() || chars[i] == '.' {
            let mut num = String::new();
            let mut has_dot = false;
            while i < chars.len() && (chars[i].is_ascii_digit() || (!has_dot && chars[i] == '.')) {
                if chars[i] == '.' {
                    has_dot = true;
                }
                num.push(chars[i]);
                i += 1;
            }
            if num == "." {
                return Err(ChainError::ToolError("Invalid number: lone decimal point".into()));
            }
            let val: f64 = num
                .parse()
                .map_err(|_| ChainError::ToolError(format!("Invalid number: {}", num)))?;
            tokens.push(Token::Number(val));
            continue;
        }

        match chars[i] {
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Star),
            '/' => tokens.push(Token::Slash),
            '^' => tokens.push(Token::Caret),
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            c if c.is_ascii_alphabetic() => {
                let mut name = String::new();
                while i < chars.len() && chars[i].is_ascii_alphabetic() {
                    name.push(chars[i]);
                    i += 1;
                }
                match name.as_str() {
                    "pi" => tokens.push(Token::Number(std::f64::consts::PI)),
                    "e" => tokens.push(Token::Number(std::f64::consts::E)),
                    "sqrt" | "sin" | "cos" | "tan" | "log" | "ln" | "abs" | "floor" | "ceil" | "round" => {
                        tokens.push(Token::Function(name));
                    }
                    _ => {
                        return Err(ChainError::ToolError(format!(
                            "Unknown function: {}",
                            name
                        )))
                    }
                }
                continue;
            }
            _ => {
                return Err(ChainError::ToolError(format!(
                    "Unexpected character: '{}'",
                    chars[i]
                )))
            }
        }
        i += 1;
    }

    Ok(tokens)
}

fn handle_unary(tokens: &mut Vec<Token>) {
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            Token::Plus
                if i == 0
                    || matches!(
                        tokens[i - 1],
                        Token::Plus
                            | Token::Minus
                            | Token::Star
                            | Token::Slash
                            | Token::Caret
                            | Token::LeftParen
                            | Token::Function(_)
                    ) =>
            {
                tokens.remove(i);
                continue;
            }
            Token::Minus
                if i == 0
                    || matches!(
                        tokens[i - 1],
                        Token::Plus
                            | Token::Minus
                            | Token::Star
                            | Token::Slash
                            | Token::Caret
                            | Token::LeftParen
                            | Token::Function(_)
                    ) =>
            {
                tokens[i] = Token::UnaryMinus;
            }
            _ => {}
        }
        i += 1;
    }
}

fn precedence(op: &Token) -> u8 {
    match op {
        Token::UnaryMinus => 5,
        Token::Caret => 4,
        Token::Star | Token::Slash => 3,
        Token::Plus | Token::Minus => 2,
        _ => 0,
    }
}

fn is_right_associative(op: &Token) -> bool {
    matches!(op, Token::Caret | Token::UnaryMinus)
}

fn shunting_yard(mut tokens: Vec<Token>) -> std::result::Result<Vec<Token>, ChainError> {
    handle_unary(&mut tokens);

    let mut output = Vec::new();
    let mut op_stack: Vec<Token> = Vec::new();

    for token in tokens {
        match token {
            Token::Number(_) => output.push(token),
            Token::Function(_) => op_stack.push(token),
            Token::UnaryMinus | Token::Plus | Token::Minus | Token::Star | Token::Slash
            | Token::Caret => {
                while let Some(top) = op_stack.last() {
                    match top {
                        Token::LeftParen | Token::Function(_) => break,
                        _ => {
                            let prec_cur = precedence(&token);
                            let prec_top = precedence(top);
                            if prec_top > prec_cur
                                || (prec_top == prec_cur && !is_right_associative(&token))
                            {
                                output.push(op_stack.pop().unwrap());
                            } else {
                                break;
                            }
                        }
                    }
                }
                op_stack.push(token);
            }
            Token::LeftParen => op_stack.push(token),
            Token::RightParen => {
                loop {
                    match op_stack.pop() {
                        Some(Token::LeftParen) => break,
                        Some(t) => output.push(t),
                        None => {
                            return Err(ChainError::ToolError(
                                "Mismatched parentheses".into(),
                            ))
                        }
                    }
                }
                if let Some(Token::Function(_)) = op_stack.last() {
                    output.push(op_stack.pop().unwrap());
                }
            }
        }
    }

    while let Some(op) = op_stack.pop() {
        if op == Token::LeftParen {
            return Err(ChainError::ToolError("Mismatched parentheses".into()));
        }
        output.push(op);
    }

    Ok(output)
}

fn evaluate_rpn(rpn: Vec<Token>) -> std::result::Result<f64, ChainError> {
    let mut stack: Vec<f64> = Vec::new();

    for token in rpn {
        match token {
            Token::Number(n) => stack.push(n),
            Token::UnaryMinus => {
                let a = stack.pop().ok_or_else(|| {
                    ChainError::ToolError("Invalid expression: stack underflow".into())
                })?;
                stack.push(-a);
            }
            Token::Plus => {
                let b = stack.pop().ok_or_else(|| {
                    ChainError::ToolError("Invalid expression: stack underflow".into())
                })?;
                let a = stack.pop().ok_or_else(|| {
                    ChainError::ToolError("Invalid expression: stack underflow".into())
                })?;
                stack.push(a + b);
            }
            Token::Minus => {
                let b = stack.pop().ok_or_else(|| {
                    ChainError::ToolError("Invalid expression: stack underflow".into())
                })?;
                let a = stack.pop().ok_or_else(|| {
                    ChainError::ToolError("Invalid expression: stack underflow".into())
                })?;
                stack.push(a - b);
            }
            Token::Star => {
                let b = stack.pop().ok_or_else(|| {
                    ChainError::ToolError("Invalid expression: stack underflow".into())
                })?;
                let a = stack.pop().ok_or_else(|| {
                    ChainError::ToolError("Invalid expression: stack underflow".into())
                })?;
                stack.push(a * b);
            }
            Token::Slash => {
                let b = stack.pop().ok_or_else(|| {
                    ChainError::ToolError("Invalid expression: stack underflow".into())
                })?;
                let a = stack.pop().ok_or_else(|| {
                    ChainError::ToolError("Invalid expression: stack underflow".into())
                })?;
                if b == 0.0 {
                    return Err(ChainError::ToolError("Division by zero".into()));
                }
                stack.push(a / b);
            }
            Token::Caret => {
                let b = stack.pop().ok_or_else(|| {
                    ChainError::ToolError("Invalid expression: stack underflow".into())
                })?;
                let a = stack.pop().ok_or_else(|| {
                    ChainError::ToolError("Invalid expression: stack underflow".into())
                })?;
                stack.push(a.powf(b));
            }
            Token::Function(name) => {
                let a = stack.pop().ok_or_else(|| {
                    ChainError::ToolError("Invalid expression: stack underflow".into())
                })?;
                let result = match name.as_str() {
                    "sqrt" => a.sqrt(),
                    "sin" => a.sin(),
                    "cos" => a.cos(),
                    "tan" => a.tan(),
                    "log" => a.log10(),
                    "ln" => a.ln(),
                    "abs" => a.abs(),
                    "floor" => a.floor(),
                    "ceil" => a.ceil(),
                    "round" => a.round(),
                    _ => {
                        return Err(ChainError::ToolError(format!(
                            "Unknown function: {}",
                            name
                        )))
                    }
                };
                stack.push(result);
            }
            _ => {
                return Err(ChainError::ToolError("Invalid token in expression".into()));
            }
        }
    }

    match stack.len() {
        1 => Ok(stack[0]),
        _ => Err(ChainError::ToolError("Invalid expression".into())),
    }
}
