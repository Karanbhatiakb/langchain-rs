//! Token encoding and decoding for counting and splitting tokens.
//!
//! Provides the [`TokenEncoder`] trait along with simple encoder
//! implementations (Cl100K, P50K, R50K) that approximate OpenAI tokenizer
//! families.

use crate::errors::*;

/// Trait for tokenizer backends that convert text to/from token IDs.
pub trait TokenEncoder: Send + Sync {
    /// Encodes a text string into a sequence of token IDs.
    fn encode(&self, text: &str) -> Result<Vec<u32>>;
    /// Decodes a sequence of token IDs back into a text string.
    fn decode(&self, tokens: &[u32]) -> Result<String>;
    /// Returns the number of tokens in the given text.
    fn get_token_count(&self, text: &str) -> Result<usize> {
        self.encode(text).map(|t| t.len())
    }
}

/// Approximates the Cl100K tokenizer used by GPT-4, GPT-4o, and GPT-3.5
/// models.
///
/// ASCII characters map directly; non-ASCII bytes are offset by 256.
#[derive(Debug, Clone)]
pub struct Cl100KEncoder;

impl Cl100KEncoder {
    /// Creates a new `Cl100KEncoder`.
    pub fn new() -> Self {
        Self
    }
}

impl TokenEncoder for Cl100KEncoder {
    fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let mut tokens = Vec::new();
        for ch in text.chars() {
            let code = ch as u32;
            if code < 128 {
                tokens.push(code);
            } else {
                let mut buf = [0u8; 4];
                let encoded = ch.encode_utf8(&mut buf);
                for byte in encoded.bytes() {
                    tokens.push(256 + byte as u32);
                }
            }
        }
        Ok(tokens)
    }

    fn decode(&self, tokens: &[u32]) -> Result<String> {
        let mut bytes = Vec::new();
        for &t in tokens {
            if t < 256 {
                bytes.push(t as u8);
            }
        }
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }
}

/// Approximates the P50K tokenizer used by GPT-3 models.
///
/// Maps each byte to a single token ID.
#[derive(Debug, Clone)]
pub struct P50KEncoder;

impl P50KEncoder {
    /// Creates a new `P50KEncoder`.
    pub fn new() -> Self {
        Self
    }
}

impl TokenEncoder for P50KEncoder {
    fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let mut tokens = Vec::new();
        for byte in text.bytes() {
            tokens.push(byte as u32);
        }
        Ok(tokens)
    }

    fn decode(&self, tokens: &[u32]) -> Result<String> {
        let bytes: Vec<u8> = tokens.iter().map(|&t| t as u8).collect();
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }
}

/// Approximates the R50K tokenizer (default fallback).
///
/// Maps each byte to a single token ID.
#[derive(Debug, Clone)]
pub struct R50KEncoder;

impl R50KEncoder {
    /// Creates a new `R50KEncoder`.
    pub fn new() -> Self {
        Self
    }
}

impl TokenEncoder for R50KEncoder {
    fn encode(&self, text: &str) -> Result<Vec<u32>> {
        let mut tokens = Vec::new();
        for byte in text.bytes() {
            tokens.push(byte as u32);
        }
        Ok(tokens)
    }

    fn decode(&self, tokens: &[u32]) -> Result<String> {
        let bytes: Vec<u8> = tokens.iter().map(|&t| t as u8).collect();
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }
}

/// Returns an appropriate token encoder for a given model name.
///
/// Heuristic mapping:
/// - GPT-4, GPT-4o, GPT-3.5 → [`Cl100KEncoder`]
/// - GPT-3 → [`P50KEncoder`]
/// - Everything else → [`R50KEncoder`]
pub fn get_encoder_for_model(model: &str) -> Box<dyn TokenEncoder> {
    let model = model.to_lowercase();
    if model.contains("gpt-4") || model.contains("gpt-3.5") || model.contains("gpt-4o") {
        Box::new(Cl100KEncoder)
    } else if model.contains("gpt-3") {
        Box::new(P50KEncoder)
    } else {
        Box::new(R50KEncoder)
    }
}
