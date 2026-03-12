pub mod token;

use logos::Logos;
use token::{Span, SpannedToken, Token};

#[derive(Debug)]
pub struct LexError {
    pub message: String,
    pub line: u32,
    pub col: u32,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Lex error at line {}, col {}: {}",
            self.line, self.col, self.message
        )
    }
}

impl std::error::Error for LexError {}

/// Tokenize a Lumina source string into a sequence of spanned tokens.
///
/// Returns `Err(LexError)` on the first unrecognised character, with
/// line and column information for diagnostics.
fn lex_raw(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut lexer = Token::lexer(source);
    let mut tokens = Vec::new();
    let mut line: u32 = 1;
    let mut col: u32 = 1;

    // We need to track position ourselves because logos skips whitespace
    // and we want accurate line/col for every token.
    let mut last_byte: usize = 0;

    while let Some(result) = lexer.next() {
        let span = lexer.span();

        // Walk through any characters between last_byte and span.start
        // to keep line/col accurate (accounts for skipped whitespace/comments).
        for byte in source[last_byte..span.start].bytes() {
            if byte == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        let token_start_col = col;
        let token_start_line = line;

        match result {
            Ok(tok) => {
                // If the token is a Newline, update line tracking
                if tok == Token::Newline {
                    tokens.push(SpannedToken {
                        token: tok,
                        span: Span {
                            start: span.start as u32,
                            end: span.end as u32,
                            line: token_start_line,
                            col: token_start_col,
                        },
                    });
                    line += 1;
                    col = 1;
                } else {
                    // Advance col by the length of the token slice
                    let token_len = span.end - span.start;
                    tokens.push(SpannedToken {
                        token: tok,
                        span: Span {
                            start: span.start as u32,
                            end: span.end as u32,
                            line: token_start_line,
                            col: token_start_col,
                        },
                    });
                    col += token_len as u32;
                }
            }
            Err(()) => {
                return Err(LexError {
                    message: format!(
                        "Unexpected character '{}'",
                        &source[span.start..span.end]
                    ),
                    line: token_start_line,
                    col: token_start_col,
                });
            }
        }

        last_byte = span.end;
    }

    Ok(tokens)
}

/// Tokenize a Lumina source string, expanding any interpolated strings.
pub fn tokenize(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    let raw = lex_raw(source)?;
    Ok(expand_interpolations(raw))
}

fn expand_interpolations(tokens: Vec<SpannedToken>) -> Vec<SpannedToken> {
    let mut out = Vec::new();
    for tok in tokens {
        if let Token::Text(ref s) = tok.token {
            if s.contains('{') {
                // Split into interpolation token sequence
                out.extend(split_interpolated(s, tok.span));
                continue;
            }
        }
        out.push(tok);
    }
    out
}

fn split_interpolated(s: &str, base_span: Span) -> Vec<SpannedToken> {
    let mut result = Vec::new();
    // Shorthand to create a SpannedToken with the same span as the parent string literal
    let spanned = |token| SpannedToken { token, span: base_span };

    result.push(spanned(Token::InterpStringStart));
    
    let mut chars = s.chars().peekable();
    let mut literal = String::new();
    
    while let Some(ch) = chars.next() {
        match ch {
            '{' if chars.peek() == Some(&'{') => {
                chars.next(); literal.push('{'); // {{ -> {
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next(); literal.push('}'); // }} -> }
            }
            '{' => {
                if !literal.is_empty() {
                    result.push(spanned(Token::InterpPart(literal.clone())));
                    literal.clear();
                }
                result.push(spanned(Token::InterpExprStart));
                
                // Collect until matching }
                let mut expr_src = String::new();
                let mut depth = 1;
                while let Some(ch2) = chars.next() {
                    if ch2 == '{' { depth += 1; }
                    if ch2 == '}' { 
                        depth -= 1; 
                        if depth == 0 { break; } 
                    }
                    expr_src.push(ch2);
                }
                
                // Re-tokenize the expression inside {}
                if let Ok(inner) = lex_raw(&expr_src) {
                    // Note: In a production compiler, we would offset these spans
                    // to point into the original file. For v1.4, we use the base_span.
                    result.extend(inner);
                }
                result.push(spanned(Token::InterpExprEnd));
            }
            c => literal.push(c),
        }
    }
    
    if !literal.is_empty() {
        result.push(spanned(Token::InterpPart(literal)));
    }
    
    result.push(spanned(Token::InterpStringEnd));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    #[test]
    fn test_entity_person() {
        let source = r#"entity Person {
  name: Text
  age: Number
  isAdult := age >= 18
}"#;

        let tokens = tokenize(source).expect("lexing should succeed");

        // Collect just the token variants for assertion (ignore spans).
        let kinds: Vec<&Token> = tokens.iter().map(|st| &st.token).collect();

        assert_eq!(
            kinds,
            vec![
                // entity Person {
                &Token::KwEntity,
                &Token::Ident("Person".into()),
                &Token::LBrace,
                &Token::Newline,
                // name: Text
                &Token::Ident("name".into()),
                &Token::Colon,
                &Token::KwTypeText,
                &Token::Newline,
                // age: Number
                &Token::Ident("age".into()),
                &Token::Colon,
                &Token::KwTypeNumber,
                &Token::Newline,
                // isAdult := age >= 18
                &Token::Ident("isAdult".into()),
                &Token::ColonEq,
                &Token::Ident("age".into()),
                &Token::GtEq,
                &Token::Number(18.0),
                &Token::Newline,
                // }
                &Token::RBrace,
            ]
        );

        // Verify first token span is correct
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.col, 1);
        assert_eq!(tokens[0].span.start, 0);
        assert_eq!(tokens[0].span.end, 6);
    }
}
