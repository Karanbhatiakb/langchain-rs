//! LLM Math chain implementation — uses LLM to generate and evaluate math expressions.

use async_trait::async_trait;
use langchain_core::errors::*;
use langchain_core::messages::HumanMessage;
use langchain_core::prompt::PromptTemplate;
use langchain_llms::traits::ChatModel;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::types::Chain;

pub struct LLMMathChain {
    llm: Arc<dyn ChatModel>,
    prompt: PromptTemplate,
    verbose: bool,
}

impl LLMMathChain {
    pub fn new(llm: Arc<dyn ChatModel>) -> Self {
        let prompt = PromptTemplate::from_template(
            "You are a math problem solver. Translate the following math question into a Python expression that can be evaluated.\n\n\
            Only output the Python expression, nothing else. Use only basic math operations and the `math` module.\n\n\
            Question: {question}\n\n\
            Python expression:",
        );
        Self {
            llm,
            prompt,
            verbose: false,
        }
    }

    pub fn with_prompt(mut self, prompt: PromptTemplate) -> Self {
        self.prompt = prompt;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    fn evaluate_expression(&self, expr: &str) -> Result<f64> {
        let cleaned = expr.trim().trim_end_matches(';').trim();

        let _unused_re = Regex::new(r"^[0-9+\-*/().%_\s]+$").unwrap();
        let sanitized: String = cleaned
            .chars()
            .filter(|c| c.is_ascii_digit() || "+-*/().%e ".contains(*c))
            .collect();

        if sanitized.is_empty() {
            return Err(ChainError::ParserError(
                "Empty expression after sanitization".to_string(),
            ));
        }

        self.eval_simple_math(&sanitized)
    }

    fn eval_simple_math(&self, expr: &str) -> Result<f64> {
        let tokens = self.tokenize(expr)?;
        let (result, _) = self.parse_expression(&tokens, 0)?;
        Ok(result)
    }

    fn tokenize(&self, expr: &str) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut chars = expr.chars().peekable();

        while let Some(&c) = chars.peek() {
            match c {
                ' ' | '\t' | '\n' => {
                    chars.next();
                }
                '+' => { chars.next(); tokens.push(Token::Plus); }
                '-' => { chars.next(); tokens.push(Token::Minus); }
                '*' => {
                    chars.next();
                    if chars.peek() == Some(&'*') {
                        chars.next();
                        tokens.push(Token::Power);
                    } else {
                        tokens.push(Token::Multiply);
                    }
                }
                '/' => { chars.next(); tokens.push(Token::Divide); }
                '%' => { chars.next(); tokens.push(Token::Modulo); }
                '(' => { chars.next(); tokens.push(Token::LParen); }
                ')' => { chars.next(); tokens.push(Token::RParen); }
                '.' | '0'..='9' | 'e' => {
                    let mut num_str = String::new();
                    while let Some(&nc) = chars.peek() {
                        if nc.is_ascii_digit() || nc == '.' || nc == 'e' || nc == '-' || nc == '+' {
                            num_str.push(nc);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    let num: f64 = num_str
                        .parse()
                        .map_err(|e| ChainError::ParserError(format!("Invalid number {}: {}", num_str, e)))?;
                    tokens.push(Token::Number(num));
                }
                _ => {
                    return Err(ChainError::ParserError(format!(
                        "Unexpected character in expression: {}",
                        c
                    )));
                }
            }
        }
        Ok(tokens)
    }

    fn parse_expression(&self, tokens: &[Token], pos: usize) -> Result<(f64, usize)> {
        let (mut result, mut pos) = self.parse_term(tokens, pos)?;

        while pos < tokens.len() {
            match &tokens[pos] {
                Token::Plus => {
                    let (right, new_pos) = self.parse_term(tokens, pos + 1)?;
                    result += right;
                    pos = new_pos;
                }
                Token::Minus => {
                    let (right, new_pos) = self.parse_term(tokens, pos + 1)?;
                    result -= right;
                    pos = new_pos;
                }
                _ => break,
            };
        }

        Ok((result, pos))
    }

    fn parse_term(&self, tokens: &[Token], pos: usize) -> Result<(f64, usize)> {
        let (mut result, mut pos) = self.parse_power(tokens, pos)?;

        while pos < tokens.len() {
            match &tokens[pos] {
                Token::Multiply => {
                    let (right, new_pos) = self.parse_power(tokens, pos + 1)?;
                    result *= right;
                    pos = new_pos;
                }
                Token::Divide => {
                    let (right, new_pos) = self.parse_power(tokens, pos + 1)?;
                    if right == 0.0 {
                        return Err(ChainError::ParserError("Division by zero".to_string()));
                    }
                    result /= right;
                    pos = new_pos;
                }
                Token::Modulo => {
                    let (right, new_pos) = self.parse_power(tokens, pos + 1)?;
                    if right == 0.0 {
                        return Err(ChainError::ParserError("Modulo by zero".to_string()));
                    }
                    result %= right;
                    pos = new_pos;
                }
                _ => break,
            };
        }

        Ok((result, pos))
    }

    fn parse_power(&self, tokens: &[Token], pos: usize) -> Result<(f64, usize)> {
        let (base, mut pos) = self.parse_unary(tokens, pos)?;

        if pos < tokens.len() && matches!(&tokens[pos], Token::Power) {
            let (exponent, new_pos) = self.parse_power(tokens, pos + 1)?;
            pos = new_pos;
            return Ok((base.powf(exponent), pos));
        }

        Ok((base, pos))
    }

    fn parse_unary(&self, tokens: &[Token], pos: usize) -> Result<(f64, usize)> {
        if pos < tokens.len() {
            if matches!(&tokens[pos], Token::Minus) {
                let (val, new_pos) = self.parse_primary(tokens, pos + 1)?;
                return Ok((-val, new_pos));
            }
            if matches!(&tokens[pos], Token::Plus) {
                return self.parse_primary(tokens, pos + 1);
            }
        }
        self.parse_primary(tokens, pos)
    }

    fn parse_primary(&self, tokens: &[Token], pos: usize) -> Result<(f64, usize)> {
        if pos >= tokens.len() {
            return Err(ChainError::ParserError("Unexpected end of expression".to_string()));
        }

        match &tokens[pos] {
            Token::Number(n) => Ok((*n, pos + 1)),
            Token::LParen => {
                let (result, new_pos) = self.parse_expression(tokens, pos + 1)?;
                if new_pos >= tokens.len() || !matches!(&tokens[new_pos], Token::RParen) {
                    return Err(ChainError::ParserError("Missing closing parenthesis".to_string()));
                }
                Ok((result, new_pos + 1))
            }
            _ => Err(ChainError::ParserError(format!(
                "Unexpected token at position {}: {:?}",
                pos, tokens[pos]
            ))),
        }
    }

    async fn get_llm_expression(&self, question: &str) -> Result<String> {
        let mut kwargs = HashMap::new();
        kwargs.insert("question".to_string(), question.to_string());
        let prompt = self.prompt.format(&kwargs)?;

        let messages = vec![HumanMessage::new(&prompt).into()];
        let response = self.llm.predict_messages(&messages, None, None).await?;

        Ok(response.content.trim().to_string())
    }
}

#[derive(Debug)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Power,
    LParen,
    RParen,
}

#[async_trait]
impl Chain for LLMMathChain {
    fn input_keys(&self) -> Vec<String> {
        vec!["question".to_string()]
    }

    fn output_keys(&self) -> Vec<String> {
        vec!["output".to_string(), "expression".to_string()]
    }

    async fn call(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        let question = inputs
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if self.verbose {
            info!("LLMMathChain processing: {}", question);
        }

        let expression = self.get_llm_expression(&question).await?;

        if self.verbose {
            info!("LLMMathChain expression: {}", expression);
        }

        match self.evaluate_expression(&expression) {
            Ok(result) => {
                let output = if result == result.floor() && result.abs() < f64::MAX / 2.0 {
                    format!("{}", result as i64)
                } else {
                    format!("{}", result)
                };
                let mut result_map = HashMap::new();
                result_map.insert("output".to_string(), Value::String(output));
                result_map.insert("expression".to_string(), Value::String(expression));
                Ok(result_map)
            }
            Err(e) => {
                let error_msg = format!("Could not evaluate expression '{}': {}", expression, e);
                let mut result_map = HashMap::new();
                result_map.insert("output".to_string(), Value::String(error_msg.clone()));
                result_map.insert("expression".to_string(), Value::String(expression));
                Ok(result_map)
            }
        }
    }
}
