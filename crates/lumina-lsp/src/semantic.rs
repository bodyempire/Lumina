use tower_lsp::lsp_types::*;
use lumina_parser::ast::*;

pub const LEGEND_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::KEYWORD,     // 0
    SemanticTokenType::VARIABLE,    // 1
    SemanticTokenType::PROPERTY,    // 2
    SemanticTokenType::FUNCTION,    // 3
    SemanticTokenType::TYPE,        // 4
    SemanticTokenType::STRING,      // 5
    SemanticTokenType::NUMBER,      // 6
];

/// Walk the source line-by-line and emit semantic tokens based on keyword/type recognition.
/// This gives editors rich syntax coloring beyond basic textmate grammars.
pub fn get_semantic_tokens(prog: &Program, src: &str) -> Vec<SemanticToken> {
    let mut tokens = Vec::new();
    let mut prev_line: u32 = 0;
    let mut prev_start: u32 = 0;

    // Collect known entity names and function names for TYPE / FUNCTION highlighting
    let mut entity_names: Vec<String> = Vec::new();
    let mut fn_names: Vec<String> = Vec::new();
    let mut rule_names: Vec<String> = Vec::new();

    for stmt in &prog.statements {
        match stmt {
            Statement::Entity(e) => entity_names.push(e.name.clone()),
            Statement::ExternalEntity(e) => entity_names.push(e.name.clone()),
            Statement::Fn(f) => fn_names.push(f.name.clone()),
            Statement::Rule(r) => rule_names.push(r.name.clone()),
            _ => {}
        }
    }

    let keywords = [
        "entity", "external", "rule", "when", "then", "end", "let", "fn",
        "if", "else", "and", "or", "not", "true", "false", "becomes",
        "for", "every", "any", "all", "show", "update", "alert", "create",
        "delete", "import", "aggregate", "over", "cooldown", "on_clear",
        "ref", "write", "times", "within", "now", "prev",
    ];

    let type_keywords = ["Number", "Text", "Boolean", "Timestamp"];

    for (line_idx, line_text) in src.lines().enumerate() {
        let line_num = line_idx as u32;
        // Tokenize the line by word boundaries
        let mut col = 0usize;
        let chars: Vec<char> = line_text.chars().collect();

        while col < chars.len() {
            // Skip whitespace
            if chars[col].is_whitespace() {
                col += 1;
                continue;
            }

            // String literal
            if chars[col] == '"' {
                let start = col;
                col += 1;
                while col < chars.len() && chars[col] != '"' {
                    if chars[col] == '\\' { col += 1; } // skip escape
                    col += 1;
                }
                if col < chars.len() { col += 1; } // closing quote
                let len = col - start;
                push_token(&mut tokens, line_num, start as u32, len as u32, 5, &mut prev_line, &mut prev_start);
                continue;
            }

            // Number literal
            if chars[col].is_ascii_digit() || (chars[col] == '-' && col + 1 < chars.len() && chars[col + 1].is_ascii_digit()) {
                let start = col;
                if chars[col] == '-' { col += 1; }
                while col < chars.len() && (chars[col].is_ascii_digit() || chars[col] == '.') {
                    col += 1;
                }
                let len = col - start;
                push_token(&mut tokens, line_num, start as u32, len as u32, 6, &mut prev_line, &mut prev_start);
                continue;
            }

            // Comment (// to end of line)
            if col + 1 < chars.len() && chars[col] == '/' && chars[col + 1] == '/' {
                break; // skip rest of line
            }

            // Identifier / keyword
            if chars[col].is_alphabetic() || chars[col] == '_' {
                let start = col;
                while col < chars.len() && (chars[col].is_alphanumeric() || chars[col] == '_') {
                    col += 1;
                }
                let word: String = chars[start..col].iter().collect();
                let len = word.len() as u32;

                let token_type = if keywords.contains(&word.as_str()) {
                    Some(0) // KEYWORD
                } else if type_keywords.contains(&word.as_str()) {
                    Some(4) // TYPE
                } else if entity_names.contains(&word) {
                    Some(4) // TYPE (entity names are types)
                } else if fn_names.contains(&word) {
                    Some(3) // FUNCTION
                } else {
                    // Check if followed by '.' — then it's likely a variable
                    if col < chars.len() && chars[col] == '.' {
                        Some(1) // VARIABLE
                    } else {
                        Some(2) // PROPERTY (field names, etc.)
                    }
                };

                if let Some(tt) = token_type {
                    push_token(&mut tokens, line_num, start as u32, len, tt, &mut prev_line, &mut prev_start);
                }
                continue;
            }

            // Skip operators and punctuation
            col += 1;
        }
    }

    tokens
}

fn push_token(
    tokens: &mut Vec<SemanticToken>,
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
    prev_line: &mut u32,
    prev_start: &mut u32,
) {
    let delta_line = line - *prev_line;
    let delta_start = if delta_line == 0 { start - *prev_start } else { start };

    tokens.push(SemanticToken {
        delta_line,
        delta_start,
        length,
        token_type,
        token_modifiers_bitset: 0,
    });

    *prev_line = line;
    *prev_start = start;
}
