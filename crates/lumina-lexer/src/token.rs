use logos::Logos;
use serde::{Serialize, Deserialize};

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    // ── Keywords ──────────────────────────────────────────
    #[token("entity")]   KwEntity,
    #[token("let")]      KwLet,
    #[token("rule")]     KwRule,
    #[token("when")]     KwWhen,
    #[token("then")]     KwThen,
    #[token("becomes")]  KwBecomes,
    #[token("for")]      KwFor,
    #[token("every")]    KwEvery,
    #[token("external")] KwExternal,
    #[token("sync")]     KwSync,
    #[token("on")]       KwOn,
    #[token("show")]     KwShow,
    #[token("update")]   KwUpdate,
    #[token("to")]       KwTo,
    #[token("create")]   KwCreate,
    #[token("delete")]   KwDelete,
    #[token("if")]       KwIf,
    #[token("else")]     KwElse,
    #[token("and")]      KwAnd,
    #[token("or")]       KwOr,
    #[token("not")]      KwNot,
    #[token("true")]     KwTrue,
    #[token("false")]    KwFalse,
    #[token("Text")]     KwTypeText,
    #[token("Number")]   KwTypeNumber,
    #[token("Boolean")]  KwTypeBoolean,
    #[token("fn")]       KwFn,
    #[token("import")]   Import,

    // ── Operators & punctuation ────────────────────────────
    #[token(":=")]  ColonEq,
    #[token(":")]   Colon,
    #[token("==")]  EqEq,
    #[token("=")]   Eq,
    #[token("!=")]  BangEq,
    #[token(">=")]  GtEq,
    #[token("<=")]  LtEq,
    #[token(">")]   Gt,
    #[token("<")]   Lt,
    #[token("+")]   Plus,
    #[token("-")]   Minus,
    #[token("*")]   Star,
    #[token("/")]   Slash,
    #[token("{")]   LBrace,
    #[token("}")]   RBrace,
    #[token("(")]   LParen,
    #[token(")")]   RParen,
    #[token(",")]   Comma,
    #[token(".")]   Dot,
    #[token("@")]   At,
    #[token("->")]  Arrow,

    // ── Literals ───────────────────────────────────────────
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    Number(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    Text(String),

    // ── Interpolation (post-processed) ─────────────────────
    InterpStringStart,
    InterpPart(String),
    InterpExprStart,
    InterpExprEnd,
    InterpStringEnd,

    // ── Identifiers ────────────────────────────────────────
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // ── Whitespace & comments ──────────────────────────────
    #[regex(r"--[^\n]*", logos::skip)]
    #[regex(r"[ \t\r]+", logos::skip)]

    #[token("\n")]
    Newline,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end:   u32,
    pub line:  u32,
    pub col:   u32,
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span:  Span,
}
