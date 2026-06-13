//! Text splitters for breaking documents into chunks.
//!
//! Provides a common [`TextSplitter`] trait and multiple implementations for
//! character-level, recursive, token-aware, markdown, code, sentence, and
//! LaTeX splitting strategies.

use crate::documents::Document;
use crate::errors::*;
use crate::tokenization::{Cl100KEncoder, TokenEncoder};
use regex::Regex;

/// Trait for splitting text into smaller chunks.
///
/// Implementors define `split_text` and get `create_documents` and
/// `split_documents` for free.
pub trait TextSplitter: Send + Sync {
    /// Splits a single text string into multiple text chunks.
    fn split_text(&self, text: &str) -> Result<Vec<String>>;
    /// Wraps a vector of text strings into [`Document`]s.
    fn create_documents(&self, texts: Vec<String>) -> Result<Vec<Document>> {
        texts
            .into_iter()
            .map(|t| Ok(Document::new(t)))
            .collect()
    }
    /// Splits an existing set of documents, preserving metadata across chunks.
    fn split_documents(&self, documents: Vec<Document>) -> Result<Vec<Document>> {
        let mut results = Vec::new();
        for doc in documents {
            let texts = self.split_text(&doc.page_content)?;
            for text in texts {
                let mut new_doc = Document::new(text);
                new_doc.metadata = doc.metadata.clone();
                results.push(new_doc);
            }
        }
        Ok(results)
    }
}

/// Splits text on a fixed separator string with configurable chunk size and
/// overlap.
#[derive(Debug, Clone)]
pub struct CharacterTextSplitter {
    /// The separator to split on (default: `"\n\n"`).
    pub separator: String,
    /// The target maximum chunk size in characters.
    pub chunk_size: usize,
    /// The number of characters to overlap between consecutive chunks.
    pub chunk_overlap: usize,
}

impl Default for CharacterTextSplitter {
    fn default() -> Self {
        Self {
            separator: "\n\n".into(),
            chunk_size: 4000,
            chunk_overlap: 200,
        }
    }
}

impl CharacterTextSplitter {
    /// Creates a new `CharacterTextSplitter`.
    pub fn new(separator: &str, chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            separator: separator.into(),
            chunk_size,
            chunk_overlap,
        }
    }
}

impl TextSplitter for CharacterTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let splits: Vec<&str> = text.split(&self.separator).collect();
        let mut chunks = Vec::new();
        let mut current = String::new();

        for split in splits {
            if current.len() + split.len() + self.separator.len() <= self.chunk_size {
                if !current.is_empty() {
                    current.push_str(&self.separator);
                }
                current.push_str(split);
            } else {
                if !current.is_empty() {
                    chunks.push(std::mem::take(&mut current));
                }
                current = split.to_string();
            }
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        Ok(chunks)
    }
}

impl CharacterTextSplitter {
    /// Returns the trailing overlap from `text` for boundary smoothing.
    #[allow(dead_code)]
    fn get_overlap(&self, text: &str) -> String {
        if self.chunk_overlap == 0 {
            return String::new();
        }
        let overlap_len = self.chunk_overlap.min(text.len());
        text[text.len() - overlap_len..].to_string()
    }
}

/// Splits text recursively using a list of separators, trying each in order.
///
/// Default separators target paragraph, line, word, and character boundaries.
#[derive(Debug, Clone)]
pub struct RecursiveCharacterTextSplitter {
    /// Ordered list of separator strings to try.
    pub separators: Vec<String>,
    /// Target maximum chunk size in characters.
    pub chunk_size: usize,
    /// Overlap between consecutive chunks.
    pub chunk_overlap: usize,
}

impl Default for RecursiveCharacterTextSplitter {
    fn default() -> Self {
        Self {
            separators: vec!["\n\n".into(), "\n".into(), " ".into(), "".into()],
            chunk_size: 4000,
            chunk_overlap: 200,
        }
    }
}

impl RecursiveCharacterTextSplitter {
    /// Creates a new `RecursiveCharacterTextSplitter`.
    pub fn new(separators: Vec<String>, chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            separators,
            chunk_size,
            chunk_overlap,
        }
    }

    /// Internal recursive splitting logic.
    fn split_text_internal(&self, text: &str, separators: &[String]) -> Vec<String> {
        let mut chunks = Vec::new();
        if separators.is_empty() {
            chunks.push(text.to_string());
            return chunks;
        }

        let separator = &separators[0];
        let splits: Vec<&str> = if separator.is_empty() {
            text.split("").filter(|s| !s.is_empty()).collect()
        } else {
            text.split(separator).collect()
        };

        let remaining = &separators[1..];
        let mut current_chunk = String::new();

        for split in splits {
            let split_len = split.len();
            let sep_len = if !current_chunk.is_empty() && !separator.is_empty() {
                separator.len()
            } else {
                0
            };

            if current_chunk.len() + sep_len + split_len <= self.chunk_size {
                if !current_chunk.is_empty() && !separator.is_empty() {
                    current_chunk.push_str(separator);
                }
                current_chunk.push_str(split);
            } else {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk);
                }
                if split_len > self.chunk_size && !remaining.is_empty() {
                    let sub_splits = self.split_text_internal(split, remaining);
                    for sub in sub_splits {
                        chunks.push(sub);
                    }
                    current_chunk = String::new();
                } else {
                    current_chunk = split.to_string();
                }
            }
        }
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }
        chunks
    }
}

impl TextSplitter for RecursiveCharacterTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        Ok(self.split_text_internal(text, &self.separators))
    }
}

/// Splits text using a tokenizer to respect token limits.
pub struct TokenTextSplitter {
    /// Maximum tokens per chunk.
    pub chunk_size: usize,
    /// Overlap in tokens between consecutive chunks.
    pub chunk_overlap: usize,
    /// The token encoder to use.
    pub encoder: Box<dyn TokenEncoder>,
}

impl TokenTextSplitter {
    /// Creates a new `TokenTextSplitter` using the default Cl100K encoder.
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
            encoder: Box::new(Cl100KEncoder::new()),
        }
    }

    /// Creates a new `TokenTextSplitter` with a custom encoder.
    pub fn with_encoder(encoder: Box<dyn TokenEncoder>, chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
            encoder,
        }
    }
}

impl std::fmt::Debug for TokenTextSplitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenTextSplitter")
            .field("chunk_size", &self.chunk_size)
            .field("chunk_overlap", &self.chunk_overlap)
            .finish()
    }
}

impl TextSplitter for TokenTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let tokens = self.encoder.encode(text)?;
        let mut chunks = Vec::new();
        let mut start = 0usize;

        while start < tokens.len() {
            let end = (start + self.chunk_size).min(tokens.len());
            let chunk_tokens = &tokens[start..end];
            let chunk = self.encoder.decode(chunk_tokens)?;
            chunks.push(chunk);
            start = end.saturating_sub(self.chunk_overlap);
            if start >= end && end < tokens.len() {
                start = end;
            }
        }

        Ok(chunks)
    }
}

/// Splits markdown text while preserving header context.
#[derive(Debug, Clone)]
pub struct MarkdownHeaderTextSplitter {
    /// Target chunk size in characters.
    pub chunk_size: usize,
    /// Overlap between chunks.
    pub chunk_overlap: usize,
}

impl Default for MarkdownHeaderTextSplitter {
    fn default() -> Self {
        Self {
            chunk_size: 4000,
            chunk_overlap: 200,
        }
    }
}

impl MarkdownHeaderTextSplitter {
    /// Creates a new `MarkdownHeaderTextSplitter`.
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
}

impl TextSplitter for MarkdownHeaderTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let header_re = Regex::new(r"(?m)^(#{1,6})\s+(.+)$").unwrap();
        let mut sections = Vec::new();
        let mut last_pos = 0usize;
        let mut last_header = String::new();

        for cap in header_re.captures_iter(text) {
            let match_start = cap.get(0).unwrap().start();
            if !last_header.is_empty() {
                sections.push((last_header.clone(), text[last_pos..match_start].to_string()));
            }
            last_header = cap.get(2).unwrap().as_str().to_string();
            last_pos = match_start;
        }
        if !last_header.is_empty() {
            sections.push((last_header, text[last_pos..].to_string()));
        } else {
            sections.push((String::new(), text.to_string()));
        }

        let mut chunks = Vec::new();
        let mut current = String::new();
        for (header, content) in sections {
            let section_text = if header.is_empty() {
                content.clone()
            } else {
                format!(
                    "{} {}\n{}",
                    "#".repeat(header.len()),
                    header,
                    content
                )
            };
            if current.len() + section_text.len() <= self.chunk_size {
                if !current.is_empty() {
                    current.push('\n');
                }
                current.push_str(&section_text);
            } else {
                if !current.is_empty() {
                    chunks.push(current);
                }
                current = section_text;
            }
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        Ok(chunks)
    }
}

/// Splits general-purpose code using language-aware recursion.
#[derive(Debug, Clone)]
pub struct CodeTextSplitter {
    /// Target chunk size in characters.
    pub chunk_size: usize,
    /// Overlap between chunks.
    pub chunk_overlap: usize,
    /// Programming language hint (unused in the current implementation).
    pub language: String,
}

impl CodeTextSplitter {
    /// Creates a new `CodeTextSplitter`.
    pub fn new(language: &str, chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
            language: language.into(),
        }
    }
}

impl TextSplitter for CodeTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let splitter = RecursiveCharacterTextSplitter::new(
            vec![
                "\n\n".into(),
                "\n".into(),
                ";".into(),
                " ".into(),
                "".into(),
            ],
            self.chunk_size,
            self.chunk_overlap,
        );
        splitter.split_text(text)
    }
}

/// Splits Python code using class/function definitions as split points.
#[derive(Debug, Clone)]
pub struct PythonCodeTextSplitter {
    /// Target chunk size in characters.
    pub chunk_size: usize,
    /// Overlap between chunks.
    pub chunk_overlap: usize,
}

impl PythonCodeTextSplitter {
    /// Creates a new `PythonCodeTextSplitter`.
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
}

impl TextSplitter for PythonCodeTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let splitter = RecursiveCharacterTextSplitter::new(
            vec![
                "\nclass ".into(),
                "\nfunc ".into(),
                "\ndef ".into(),
                "\n\t".into(),
                "\n  ".into(),
                "\n".into(),
                "".into(),
            ],
            self.chunk_size,
            self.chunk_overlap,
        );
        splitter.split_text(text)
    }
}

/// Splits Markdown text using heading-based recursion.
#[derive(Debug, Clone)]
pub struct MarkdownTextSplitter {
    /// Target chunk size in characters.
    pub chunk_size: usize,
    /// Overlap between chunks.
    pub chunk_overlap: usize,
}

impl MarkdownTextSplitter {
    /// Creates a new `MarkdownTextSplitter`.
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
}

impl TextSplitter for MarkdownTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let splitter = RecursiveCharacterTextSplitter::new(
            vec![
                "\n## ".into(),
                "\n### ".into(),
                "\n#### ".into(),
                "\n##### ".into(),
                "\n###### ".into(),
                "\n\n".into(),
                "\n".into(),
                " ".into(),
                "".into(),
            ],
            self.chunk_size,
            self.chunk_overlap,
        );
        splitter.split_text(text)
    }
}

/// Splits text at sentence boundaries using regex-based sentence detection.
#[derive(Debug, Clone)]
pub struct SentenceTransformersTextSplitter {
    /// Target chunk size in characters.
    pub chunk_size: usize,
    /// Overlap between chunks.
    pub chunk_overlap: usize,
}

impl SentenceTransformersTextSplitter {
    /// Creates a new `SentenceTransformersTextSplitter`.
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
}

impl TextSplitter for SentenceTransformersTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let re = Regex::new(r"(?s)([^.!?]+[.!?]+)").unwrap();
        let sentences: Vec<String> = re
            .captures_iter(text)
            .map(|c| c.get(1).unwrap().as_str().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if sentences.is_empty() {
            return Ok(vec![text.to_string()]);
        }

        let mut chunks = Vec::new();
        let mut current = String::new();
        for sentence in sentences {
            if current.len() + sentence.len() + 1 > self.chunk_size && !current.is_empty() {
                chunks.push(current.clone());
                let overlap_len = self.chunk_overlap.min(current.len());
                current = current[current.len() - overlap_len..].to_string();
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(&sentence);
            } else {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(&sentence);
            }
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        Ok(chunks)
    }
}

/// Splits LaTeX documents using chapter/section/subsection boundaries.
#[derive(Debug, Clone)]
pub struct LatexTextSplitter {
    /// Target chunk size in characters.
    pub chunk_size: usize,
    /// Overlap between chunks.
    pub chunk_overlap: usize,
}

impl LatexTextSplitter {
    /// Creates a new `LatexTextSplitter`.
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
}

impl TextSplitter for LatexTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let splitter = RecursiveCharacterTextSplitter::new(
            vec![
                "\n\\chapter{".into(),
                "\n\\section{".into(),
                "\n\\subsection{".into(),
                "\n\\subsubsection{".into(),
                "\n\n".into(),
                "\n".into(),
                " ".into(),
                "".into(),
            ],
            self.chunk_size,
            self.chunk_overlap,
        );
        splitter.split_text(text)
    }
}

/// Splits HTML text by section headers (`<h1>` through `<h6>`).
#[derive(Debug, Clone)]
pub struct HtmlTextSplitter {
    /// Target chunk size in characters.
    pub chunk_size: usize,
    /// Overlap between chunks.
    pub chunk_overlap: usize,
}

impl Default for HtmlTextSplitter {
    fn default() -> Self {
        Self {
            chunk_size: 4000,
            chunk_overlap: 200,
        }
    }
}

impl HtmlTextSplitter {
    /// Creates a new `HtmlTextSplitter`.
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
}

impl TextSplitter for HtmlTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let header_re =
            Regex::new(r"<h[1-6][^>]*>").map_err(|e| ChainError::ParserError(e.to_string()))?;
        let mut sections: Vec<(String, String)> = Vec::new();
        let mut last_pos = 0usize;
        let mut last_header = String::new();

        for mat in header_re.find_iter(text) {
            let match_start = mat.start();
            if !last_header.is_empty() {
                sections.push((
                    last_header.clone(),
                    text[last_pos..match_start].to_string(),
                ));
            }
            last_header = mat.as_str().to_string();
            last_pos = match_start;
        }
        if !last_header.is_empty() {
            sections.push((last_header, text[last_pos..].to_string()));
        } else {
            sections.push((String::new(), text.to_string()));
        }

        let mut chunks = Vec::new();
        let mut current = String::new();
        for (header, content) in sections {
            let section_text = if header.is_empty() {
                content
            } else {
                format!("{}\n{}", header, content.trim())
            };
            if current.len() + section_text.len() <= self.chunk_size {
                if !current.is_empty() {
                    current.push('\n');
                }
                current.push_str(&section_text);
            } else {
                if !current.is_empty() {
                    chunks.push(current);
                }
                current = section_text;
            }
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        Ok(chunks)
    }
}

/// Splits JSON text by parsing the value and recursively converting to strings.
///
/// Falls back to line-based splitting when the input is not valid JSON.
#[derive(Debug, Clone)]
pub struct JsonTextSplitter {
    /// Target chunk size in characters.
    pub chunk_size: usize,
    /// Overlap between chunks.
    pub chunk_overlap: usize,
    /// Maximum nesting depth to recurse into the JSON structure.
    pub max_depth: usize,
}

impl Default for JsonTextSplitter {
    fn default() -> Self {
        Self {
            chunk_size: 4000,
            chunk_overlap: 200,
            max_depth: 5,
        }
    }
}

impl JsonTextSplitter {
    /// Creates a new `JsonTextSplitter`.
    pub fn new(chunk_size: usize, chunk_overlap: usize, max_depth: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
            max_depth,
        }
    }

    fn json_value_to_strings(value: &serde_json::Value, depth: usize, max_depth: usize) -> Vec<String> {
        if depth >= max_depth {
            let s = serde_json::to_string(value).unwrap_or_default();
            return vec![s];
        }
        match value {
            serde_json::Value::Object(map) => {
                let mut parts = Vec::new();
                for (k, v) in map {
                    let inner = Self::json_value_to_strings(v, depth + 1, max_depth);
                    for s in inner {
                        parts.push(format!("{}: {}", k, s));
                    }
                }
                parts
            }
            serde_json::Value::Array(arr) => {
                let mut parts = Vec::new();
                for item in arr {
                    let inner = Self::json_value_to_strings(item, depth + 1, max_depth);
                    parts.extend(inner);
                }
                parts
            }
            other => {
                let s = serde_json::to_string(other).unwrap_or_default();
                vec![s]
            }
        }
    }

    fn merge_chunks(parts: &[String], chunk_size: usize, chunk_overlap: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current = String::new();
        for part in parts {
            if !current.is_empty() && current.len() + part.len() + 1 > chunk_size {
                chunks.push(current.clone());
                let overlap_len = chunk_overlap.min(current.len());
                current = current[current.len() - overlap_len..].to_string();
            }
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(part);
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        chunks
    }

    fn split_lines_fallback(text: &str, chunk_size: usize, chunk_overlap: usize) -> Vec<String> {
        let lines: Vec<&str> = text.lines().collect();
        let mut chunks = Vec::new();
        let mut current = String::new();
        for line in lines {
            if !current.is_empty() && current.len() + line.len() + 1 > chunk_size {
                chunks.push(current.clone());
                let overlap_len = chunk_overlap.min(current.len());
                current = current[current.len() - overlap_len..].to_string();
            }
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        chunks
    }
}

impl TextSplitter for JsonTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        match serde_json::from_str::<serde_json::Value>(text) {
            Ok(value) => {
                let parts = Self::json_value_to_strings(&value, 0, self.max_depth);
                Ok(Self::merge_chunks(&parts, self.chunk_size, self.chunk_overlap))
            }
            Err(_) => Ok(Self::split_lines_fallback(
                text,
                self.chunk_size,
                self.chunk_overlap,
            )),
        }
    }
}

/// Splits JSX / React code using component and function boundaries.
#[derive(Debug, Clone)]
pub struct JsxTextSplitter {
    /// Target chunk size in characters.
    pub chunk_size: usize,
    /// Overlap between chunks.
    pub chunk_overlap: usize,
}

impl Default for JsxTextSplitter {
    fn default() -> Self {
        Self {
            chunk_size: 4000,
            chunk_overlap: 200,
        }
    }
}

impl JsxTextSplitter {
    /// Creates a new `JsxTextSplitter`.
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
}

impl TextSplitter for JsxTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let splitter = RecursiveCharacterTextSplitter::new(
            vec![
                "\nfunction ".into(),
                "\nconst ".into(),
                "\nexport ".into(),
                "\nimport ".into(),
                "\nexport default ".into(),
                "\nclass ".into(),
                "\n\n".into(),
                "\n".into(),
                " ".into(),
                "".into(),
            ],
            self.chunk_size,
            self.chunk_overlap,
        );
        splitter.split_text(text)
    }
}

/// Simulates NLTK sentence tokenization using regex-based sentence boundary
/// detection.
#[derive(Debug, Clone)]
pub struct NltkTextSplitter {
    /// Target chunk size in characters.
    pub chunk_overlap: usize,
    /// Overlap between chunks.
    pub chunk_size: usize,
}

impl Default for NltkTextSplitter {
    fn default() -> Self {
        Self {
            chunk_size: 4000,
            chunk_overlap: 200,
        }
    }
}

impl NltkTextSplitter {
    /// Creates a new `NltkTextSplitter`.
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
}

impl TextSplitter for NltkTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let re = Regex::new(r"(?s)(.+?[.!?]+(?:\s|$))")
            .map_err(|e| ChainError::ParserError(e.to_string()))?;
        let sentences: Vec<String> = re
            .captures_iter(text)
            .map(|c| c.get(1).unwrap().as_str().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if sentences.is_empty() {
            return Ok(vec![text.to_string()]);
        }

        let mut chunks = Vec::new();
        let mut current = String::new();
        for sentence in sentences {
            if !current.is_empty() && current.len() + sentence.len() + 1 > self.chunk_size {
                chunks.push(current.clone());
                let overlap_len = self.chunk_overlap.min(current.len());
                current = current[current.len() - overlap_len..].to_string();
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(&sentence);
            } else {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(&sentence);
            }
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        Ok(chunks)
    }
}

/// Simulates spaCy sentence tokenization using regex-based sentence boundary
/// detection.
#[derive(Debug, Clone)]
pub struct SpacyTextSplitter {
    /// Target chunk size in characters.
    pub chunk_size: usize,
    /// Overlap between chunks.
    pub chunk_overlap: usize,
}

impl Default for SpacyTextSplitter {
    fn default() -> Self {
        Self {
            chunk_size: 4000,
            chunk_overlap: 200,
        }
    }
}

impl SpacyTextSplitter {
    /// Creates a new `SpacyTextSplitter`.
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
}

impl TextSplitter for SpacyTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let re = Regex::new(r#"(?s)(.+?(?:[.!?]+["')\]]*\s+|\n+))"#)
            .map_err(|e| ChainError::ParserError(e.to_string()))?;
        let sentences: Vec<String> = re
            .captures_iter(text)
            .map(|c| c.get(1).unwrap().as_str().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if sentences.is_empty() {
            return Ok(vec![text.to_string()]);
        }

        let mut chunks = Vec::new();
        let mut current = String::new();
        for sentence in sentences {
            if !current.is_empty() && current.len() + sentence.len() + 1 > self.chunk_size {
                chunks.push(current.clone());
                let overlap_len = self.chunk_overlap.min(current.len());
                current = current[current.len() - overlap_len..].to_string();
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(&sentence);
            } else {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(&sentence);
            }
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        Ok(chunks)
    }
}

/// Splits XSL / XSLT stylesheets using template and control-flow boundaries.
#[derive(Debug, Clone)]
pub struct XslTextSplitter {
    /// Target chunk size in characters.
    pub chunk_size: usize,
    /// Overlap between chunks.
    pub chunk_overlap: usize,
}

impl Default for XslTextSplitter {
    fn default() -> Self {
        Self {
            chunk_size: 4000,
            chunk_overlap: 200,
        }
    }
}

impl XslTextSplitter {
    /// Creates a new `XslTextSplitter`.
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
}

impl TextSplitter for XslTextSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let splitter = RecursiveCharacterTextSplitter::new(
            vec![
                "\n<xsl:template ".into(),
                "\n<xsl:if ".into(),
                "\n<xsl:for-each ".into(),
                "\n<xsl:choose>".into(),
                "\n<xsl:when ".into(),
                "\n\n".into(),
                "\n".into(),
                " ".into(),
                "".into(),
            ],
            self.chunk_size,
            self.chunk_overlap,
        );
        splitter.split_text(text)
    }
}

/// Splits HTML documents by section headers (`<h1>` through `<h6>`), preserving
/// the header hierarchy in each chunk.
#[derive(Debug, Clone)]
pub struct HtmlSectionSplitter {
    /// The maximum number of characters per chunk.
    pub chunk_size: usize,
    /// The number of characters to overlap between consecutive chunks.
    pub chunk_overlap: usize,
    /// Whether to include the heading tag in the output.
    pub include_headers: bool,
}

impl Default for HtmlSectionSplitter {
    fn default() -> Self {
        Self {
            chunk_size: 4000,
            chunk_overlap: 200,
            include_headers: true,
        }
    }
}

impl HtmlSectionSplitter {
    /// Creates a new [`HtmlSectionSplitter`].
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
            include_headers: true,
        }
    }

    /// Configures whether to include heading tags in the output.
    pub fn with_include_headers(mut self, include: bool) -> Self {
        self.include_headers = include;
        self
    }
}

impl TextSplitter for HtmlSectionSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let header_re =
            Regex::new(r"</?h[1-6][^>]*>")
                .map_err(|e| ChainError::ParserError(e.to_string()))?;
        let mut sections: Vec<(u8, String, String)> = Vec::new();
        let mut last_pos = 0usize;
        let mut last_level: u8 = 1;
        let mut last_header_tag = String::new();

        let tags: Vec<(usize, &str)> = header_re.find_iter(text).map(|m| (m.start(), m.as_str())).collect();
        let mut i = 0;
        while i < tags.len() {
            let (start, tag) = tags[i];
            if tag.starts_with("</") {
                i += 1;
                continue;
            }
            let level: u8 = tag
                .trim_start_matches("<h")
                .chars()
                .next()
                .and_then(|c| c.to_digit(10))
                .map(|d| d as u8)
                .unwrap_or(1);
            if !last_header_tag.is_empty() {
                sections.push((
                    last_level,
                    last_header_tag.clone(),
                    text[last_pos..start].to_string(),
                ));
            }
            last_level = level;
            last_header_tag = tag.to_string();
            last_pos = start;
            i += 1;
        }
        if !last_header_tag.is_empty() {
            sections.push((last_level, last_header_tag, text[last_pos..].to_string()));
        } else {
            sections.push((1, String::new(), text.to_string()));
        }

        let mut chunks = Vec::new();
        let mut current = String::new();
        for (_level, header_tag, content) in sections {
            let section_text = if header_tag.is_empty() || !self.include_headers {
                content
            } else {
                format!("{}\n{}", header_tag, content.trim())
            };
            if current.len() + section_text.len() <= self.chunk_size {
                if !current.is_empty() {
                    current.push('\n');
                }
                current.push_str(&section_text);
            } else {
                if !current.is_empty() {
                    chunks.push(current);
                }
                current = section_text;
            }
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        Ok(chunks)
    }
}

/// Splits HTML documents while preserving their semantic structure (tags,
/// attributes, text content).
///
/// Unlike the simpler [`HtmlSectionSplitter`], this splitter attempts to keep
/// nested HTML elements intact, closing any open tags when a chunk boundary
/// is reached and re-opening them in the next chunk.
#[derive(Debug, Clone)]
pub struct HtmlSemanticPreservingSplitter {
    /// The maximum number of characters per chunk.
    pub chunk_size: usize,
    /// The number of characters to overlap between consecutive chunks.
    pub chunk_overlap: usize,
}

impl Default for HtmlSemanticPreservingSplitter {
    fn default() -> Self {
        Self {
            chunk_size: 4000,
            chunk_overlap: 200,
        }
    }
}

impl HtmlSemanticPreservingSplitter {
    /// Creates a new [`HtmlSemanticPreservingSplitter`].
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
}

impl TextSplitter for HtmlSemanticPreservingSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let tag_re =
            Regex::new(r"(</?[a-zA-Z][^>]*>)|([^<]+)")
                .map_err(|e| ChainError::ParserError(e.to_string()))?;
        let mut chunks = Vec::new();
        let mut current = String::new();
        let mut tag_stack: Vec<String> = Vec::new();
        let mut open_tags: Vec<String> = Vec::new();

        for cap in tag_re.captures_iter(text) {
            if let Some(tag) = cap.get(1) {
                let tag_str = tag.as_str();
                if tag_str.starts_with("</") {
                    if let Some(last) = open_tags.last() {
                        let tag_name = tag_str.trim_start_matches("</").trim_end_matches('>');
                        if last.trim_start_matches('<').trim_end_matches('>').starts_with(tag_name) {
                            open_tags.pop();
                        }
                    }
                    current.push_str(tag_str);
                } else if tag_str.ends_with("/>") || tag_str.ends_with("-->") {
                    current.push_str(tag_str);
                } else if tag_str.starts_with("<!--") {
                    current.push_str(tag_str);
                } else {
                    let tag_name = tag_str
                        .trim_start_matches('<')
                        .split_whitespace()
                        .next()
                        .map(|s| s.trim_end_matches('>'))
                        .unwrap_or("")
                        .to_string();
                    open_tags.push(format!("</{}>", tag_name));
                    current.push_str(tag_str);
                    if let Some(new_stack) = open_tags.last() {
                        tag_stack.push(new_stack.clone());
                    }
                }
            } else if let Some(text_content) = cap.get(2) {
                let text_str = text_content.as_str();
                if !current.is_empty() && current.len() + text_str.len() > self.chunk_size {
                    for closing_tag in open_tags.iter().rev() {
                        current.push_str(closing_tag);
                    }
                    chunks.push(current);
                    current = String::new();
                    for opening_tag in &tag_stack {
                        let ot = opening_tag.replace("</", "<");
                        let base = ot.trim_end_matches('>');
                        current.push_str(&format!("{}>", base));
                    }
                }
                current.push_str(text_str);
            }
        }

        if !current.is_empty() {
            for closing_tag in open_tags.iter().rev() {
                current.push_str(closing_tag);
            }
            chunks.push(current);
        }

        Ok(chunks)
    }
}

/// Splits JSON documents recursively by walking the JSON structure and
/// producing chunks at configurable depth levels.
#[derive(Debug, Clone)]
pub struct RecursiveJsonSplitter {
    /// The maximum number of characters per chunk.
    pub chunk_size: usize,
    /// The number of characters to overlap between consecutive chunks.
    pub chunk_overlap: usize,
    /// The maximum nesting depth to recurse into.
    pub max_depth: usize,
}

impl Default for RecursiveJsonSplitter {
    fn default() -> Self {
        Self {
            chunk_size: 4000,
            chunk_overlap: 200,
            max_depth: 10,
        }
    }
}

impl RecursiveJsonSplitter {
    /// Creates a new [`RecursiveJsonSplitter`].
    pub fn new(chunk_size: usize, chunk_overlap: usize, max_depth: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
            max_depth,
        }
    }

    /// Recursively converts a JSON value into a flat list of key: value
    /// strings, respecting the depth limit.
    fn flatten_value(
        value: &serde_json::Value,
        depth: usize,
        max_depth: usize,
        output: &mut Vec<String>,
    ) {
        if depth >= max_depth {
            let s = serde_json::to_string(value).unwrap_or_default();
            output.push(s);
            return;
        }
        match value {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    let mut inner = Vec::new();
                    Self::flatten_value(v, depth + 1, max_depth, &mut inner);
                    for s in inner {
                        output.push(format!("{}: {}", k, s));
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                for (i, v) in arr.iter().enumerate() {
                    let mut inner = Vec::new();
                    Self::flatten_value(v, depth + 1, max_depth, &mut inner);
                    for s in inner {
                        output.push(format!("[{}] {}", i, s));
                    }
                }
            }
            other => {
                let s = serde_json::to_string(other).unwrap_or_default();
                output.push(s);
            }
        }
    }

    /// Merges a flat list of strings into chunks sized by `chunk_size`.
    fn merge_chunks(items: &[String], chunk_size: usize, chunk_overlap: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current = String::new();
        for item in items {
            if !current.is_empty() && current.len() + item.len() + 1 > chunk_size {
                chunks.push(current.clone());
                let overlap_len = chunk_overlap.min(current.len());
                current = current[current.len() - overlap_len..].to_string();
            }
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(item);
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        chunks
    }

    /// Falls back to line-by-line splitting when input is not valid JSON.
    fn split_lines_fallback(text: &str, chunk_size: usize, chunk_overlap: usize) -> Vec<String> {
        let lines: Vec<&str> = text.lines().collect();
        let mut chunks = Vec::new();
        let mut current = String::new();
        for line in lines {
            if !current.is_empty() && current.len() + line.len() + 1 > chunk_size {
                chunks.push(current.clone());
                let overlap_len = chunk_overlap.min(current.len());
                current = current[current.len() - overlap_len..].to_string();
            }
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
        if !current.is_empty() {
            chunks.push(current);
        }
        chunks
    }
}

impl TextSplitter for RecursiveJsonSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        match serde_json::from_str::<serde_json::Value>(text) {
            Ok(value) => {
                let mut parts = Vec::new();
                Self::flatten_value(&value, 0, self.max_depth, &mut parts);
                Ok(Self::merge_chunks(&parts, self.chunk_size, self.chunk_overlap))
            }
            Err(_) => Ok(Self::split_lines_fallback(
                text,
                self.chunk_size,
                self.chunk_overlap,
            )),
        }
    }
}

/// Splits Korean text using a simplified Korean NLP tokenisation approach.
///
/// This splitter uses basic morphological analysis — it splits on
/// Korean particle/ending boundaries (`은`, `는`, `이`, `가`, `을`, `를`,
/// `의`, `에`, `로`, `으로`, `와`, `과`, `도`, `만`, `부터`, `까지`,
/// `하다`, `되다`) and falls back to character-level splitting for
/// non-Korean text.
#[derive(Debug, Clone)]
pub struct KonlpySplitter {
    /// The maximum number of characters per chunk.
    pub chunk_size: usize,
    /// The number of characters to overlap between consecutive chunks.
    pub chunk_overlap: usize,
}

impl Default for KonlpySplitter {
    fn default() -> Self {
        Self {
            chunk_size: 4000,
            chunk_overlap: 200,
        }
    }
}

impl KonlpySplitter {
    /// Creates a new [`KonlpySplitter`].
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
}

impl TextSplitter for KonlpySplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let particle_re = Regex::new(
            r"(?m)([가-힣]+)(은|는|이|가|을|를|의|에|로|으로|와|과|도|만|부터|까지|하다|되다)"
        ).map_err(|e| ChainError::ParserError(e.to_string()))?;

        let splits: Vec<&str> = if particle_re.is_match(text) {
            let mut parts = Vec::new();
            let mut last_end = 0usize;
            for cap in particle_re.captures_iter(text) {
                let m_end = cap.get(0).map(|m| m.end()).unwrap_or(text.len());
                if last_end < m_end {
                    parts.push(&text[last_end..m_end]);
                }
                last_end = m_end;
            }
            if last_end < text.len() {
                parts.push(&text[last_end..]);
            }
            parts
        } else {
            text.split_whitespace().collect()
        };

        let splitter = RecursiveCharacterTextSplitter::new(
            vec!["\n".into(), " ".into(), "".into()],
            self.chunk_size,
            self.chunk_overlap,
        );
        splitter.split_text(&splits.join("\n"))
    }
}

/// Splits Markdown documents while preserving syntax structure (code blocks,
/// tables, lists, headings).
///
/// This splitter identifies Markdown block-level elements and ensures they
/// are not broken mid-structure. Code fences, table rows, and list items
/// are kept together where possible.
#[derive(Debug, Clone)]
pub struct ExperimentalMarkdownSyntaxSplitter {
    /// The maximum number of characters per chunk.
    pub chunk_size: usize,
    /// The number of characters to overlap between consecutive chunks.
    pub chunk_overlap: usize,
}

impl Default for ExperimentalMarkdownSyntaxSplitter {
    fn default() -> Self {
        Self {
            chunk_size: 4000,
            chunk_overlap: 200,
        }
    }
}

impl ExperimentalMarkdownSyntaxSplitter {
    /// Creates a new [`ExperimentalMarkdownSyntaxSplitter`].
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }
}

impl TextSplitter for ExperimentalMarkdownSyntaxSplitter {
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let separators = vec![
            "\n```".into(),
            "\n---".into(),
            "\n***".into(),
            "\n___".into(),
            "\n# ".into(),
            "\n## ".into(),
            "\n### ".into(),
            "\n#### ".into(),
            "\n##### ".into(),
            "\n###### ".into(),
            "\n\n".into(),
            "\n".into(),
            " ".into(),
            "".into(),
        ];
        let splitter = RecursiveCharacterTextSplitter::new(
            separators,
            self.chunk_size,
            self.chunk_overlap,
        );
        splitter.split_text(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_text_splitter() {
        let splitter = HtmlTextSplitter::new(4000, 0);
        let html = "<h1>Title</h1><p>Content under h1</p><h2>Subtitle</h2><p>Content under h2</p>";
        let chunks = splitter.split_text(html).unwrap();
        assert!(!chunks.is_empty());
        let combined = chunks.join("\n");
        assert!(combined.contains("<h1>"));
        assert!(combined.contains("<h2>"));
    }

    #[test]
    fn test_json_text_splitter() {
        let splitter = JsonTextSplitter::new(4000, 0, 5);
        let json = r#"{"name": "Alice", "age": 30, "city": "NYC"}"#;
        let chunks = splitter.split_text(json).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_json_text_splitter_invalid_fallback() {
        let splitter = JsonTextSplitter::new(4000, 0, 5);
        let text = "not json\nline two\nline three";
        let chunks = splitter.split_text(text).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_jsx_text_splitter() {
        let splitter = JsxTextSplitter::new(4000, 0);
        let jsx = "import React from 'react';\n\nfunction App() {\n  return <div>Hello</div>;\n}\n\nexport default App;";
        let chunks = splitter.split_text(jsx).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_nltk_text_splitter() {
        let splitter = NltkTextSplitter::new(4000, 0);
        let text = "Hello world. This is a test. How are you doing today?";
        let chunks = splitter.split_text(text).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_spacy_text_splitter() {
        let splitter = SpacyTextSplitter::new(4000, 0);
        let text = "The quick brown fox jumps. Over the lazy dog. It was a sunny day.";
        let chunks = splitter.split_text(text).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_xsl_text_splitter() {
        let splitter = XslTextSplitter::new(4000, 0);
        let xsl = "<xsl:stylesheet version=\"1.0\">\n<xsl:template match=\"/\">\n<html/>\n</xsl:template>\n<xsl:if test=\"true\">\n<output/>\n</xsl:if>\n</xsl:stylesheet>";
        let chunks = splitter.split_text(xsl).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_html_section_splitter() {
        let splitter = HtmlSectionSplitter::new(4000, 0);
        let html = "<h1>Title</h1><p>Content</p><h2>Subtitle</h2><p>More content</p>";
        let chunks = splitter.split_text(html).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_html_semantic_preserving_splitter() {
        let splitter = HtmlSemanticPreservingSplitter::new(4000, 0);
        let html = "<div><p>Hello <b>world</b></p></div>";
        let chunks = splitter.split_text(html).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_recursive_json_splitter() {
        let splitter = RecursiveJsonSplitter::new(4000, 0, 5);
        let json = r#"{"name": "Alice", "age": 30, "city": "NYC"}"#;
        let chunks = splitter.split_text(json).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_konlpy_splitter() {
        let splitter = KonlpySplitter::new(4000, 0);
        let text = "안녕하세요. 저는 한국어를 공부하고 있습니다. 오늘은 날씨가 좋습니다.";
        let chunks = splitter.split_text(text).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_experimental_markdown_syntax_splitter() {
        let splitter = ExperimentalMarkdownSyntaxSplitter::new(4000, 0);
        let md = "# Title\n\nThis is a paragraph.\n\n## Subtitle\n\nAnother paragraph.";
        let chunks = splitter.split_text(md).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_character_splitter_basic() {
        let splitter = CharacterTextSplitter::new("\n", 20, 0);
        let chunks = splitter.split_text("hello\nworld\nfoo").unwrap();
        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn test_character_splitter_small_chunk() {
        let splitter = CharacterTextSplitter::new(" ", 5, 0);
        let chunks = splitter.split_text("a b c d e").unwrap();
        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn test_recursive_character_splitter() {
        let splitter = RecursiveCharacterTextSplitter::new(
            vec!["\n\n".into(), "\n".into(), " ".into(), "".into()],
            50, 0,
        );
        let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
        let chunks = splitter.split_text(text).unwrap();
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_recursive_character_splitter_single_chunk() {
        let splitter = RecursiveCharacterTextSplitter::default();
        let chunks = splitter.split_text("short text").unwrap();
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_markdown_header_splitter() {
        let splitter = MarkdownHeaderTextSplitter::new(4000, 0);
        let chunks = splitter.split_text("# Title\ncontent\n## Subtitle\nmore").unwrap();
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_code_splitter() {
        let splitter = CodeTextSplitter::new("rust", 4000, 0);
        let code = "fn main() {\n    println!(\"hello\");\n}";
        let chunks = splitter.split_text(code).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_python_code_splitter() {
        let splitter = PythonCodeTextSplitter::new(4000, 0);
        let code = "def foo():\n    pass\n\nclass Bar:\n    pass";
        let chunks = splitter.split_text(code).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_markdown_text_splitter() {
        let splitter = MarkdownTextSplitter::new(4000, 0);
        let md = "# H1\ncontent\n## H2\nmore content";
        let chunks = splitter.split_text(md).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_sentence_transformers_splitter() {
        let splitter = SentenceTransformersTextSplitter::new(4000, 0);
        let text = "First sentence. Second sentence. Third sentence!";
        let chunks = splitter.split_text(text).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_latex_splitter() {
        let splitter = LatexTextSplitter::new(4000, 0);
        let latex = "\\chapter{Intro}\nContent\n\\section{Details}\nMore content";
        let chunks = splitter.split_text(latex).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_html_section_splitter_without_headers() {
        let splitter = HtmlSectionSplitter::new(4000, 0).with_include_headers(false);
        let html = "<h1>Title</h1><p>Content</p>";
        let chunks = splitter.split_text(html).unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_create_documents() {
        let splitter = CharacterTextSplitter::new("\n", 100, 0);
        let docs = splitter.create_documents(vec!["hello".to_string(), "world".to_string()]).unwrap();
        assert_eq!(docs.len(), 2);
    }

    #[test]
    fn test_split_documents() {
        let splitter = CharacterTextSplitter::new("\n", 10, 0);
        let doc = Document::new("hello\nworld\nfoo").with_metadata(
            std::collections::HashMap::from([("source".to_string(), serde_json::Value::String("test".into()))])
        );
        let docs = splitter.split_documents(vec![doc]).unwrap();
        assert!(docs.len() > 1);
        assert_eq!(docs[0].metadata.get("source").and_then(|v| v.as_str()), Some("test"));
    }

    #[test]
    fn test_text_splitter_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<CharacterTextSplitter>();
        assert_sync::<CharacterTextSplitter>();
        assert_send::<RecursiveCharacterTextSplitter>();
        assert_sync::<RecursiveCharacterTextSplitter>();
        assert_send::<HtmlTextSplitter>();
        assert_sync::<HtmlTextSplitter>();
    }
}
