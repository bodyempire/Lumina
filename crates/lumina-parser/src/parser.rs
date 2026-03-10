use lumina_lexer::token::{Token, SpannedToken, Span};
use crate::ast::*;
use crate::error::ParseError;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos:    usize,
}

// ── Binding-power tables ───────────────────────────────────────────────────

fn infix_bp(token: &Token) -> Option<(u8, u8)> {
    match token {
        Token::KwOr                          => Some((1, 2)),
        Token::KwAnd                         => Some((3, 4)),
        Token::EqEq | Token::BangEq          => Some((5, 6)),
        Token::Gt | Token::Lt
        | Token::GtEq | Token::LtEq          => Some((7, 8)),
        Token::Plus | Token::Minus            => Some((9, 10)),
        Token::Star | Token::Slash            => Some((11, 12)),
        Token::Dot                            => Some((17, 18)),
        _                                     => None,
    }
}

fn prefix_bp(token: &Token) -> Option<u8> {
    match token {
        Token::Minus | Token::KwNot => Some(13),
        _                           => None,
    }
}

fn token_to_binop(token: &Token) -> BinOp {
    match token {
        Token::Plus   => BinOp::Add,
        Token::Minus  => BinOp::Sub,
        Token::Star   => BinOp::Mul,
        Token::Slash  => BinOp::Div,
        Token::EqEq   => BinOp::Eq,
        Token::BangEq => BinOp::Ne,
        Token::Gt     => BinOp::Gt,
        Token::Lt     => BinOp::Lt,
        Token::GtEq   => BinOp::Ge,
        Token::LtEq   => BinOp::Le,
        Token::KwAnd  => BinOp::And,
        Token::KwOr   => BinOp::Or,
        _ => unreachable!("not a binary operator: {:?}", token),
    }
}

// ── Parser impl ────────────────────────────────────────────────────────────

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    // ── helpers ────────────────────────────────────────────

    fn current(&self) -> &Token {
        &self.tokens[self.pos].token
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1).map(|st| &st.token)
    }

    fn current_span(&self) -> Span {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].span
        } else if let Some(last) = self.tokens.last() {
            last.span
        } else {
            Span::default()
        }
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, token: &Token) -> Result<Span, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::new(
                format!("expected {:?}, got end of input", token),
                self.current_span(),
            ));
        }
        if self.current() == token {
            let span = self.current_span();
            self.advance();
            Ok(span)
        } else {
            Err(ParseError::new(
                format!("expected {:?}, got {:?}", token, self.current()),
                self.current_span(),
            ))
        }
    }

    fn check(&self, token: &Token) -> bool {
        !self.is_at_end() && self.current() == token
    }

    fn skip_newlines(&mut self) {
        while !self.is_at_end() && self.check(&Token::Newline) {
            self.advance();
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    // ── public entry ──────────────────────────────────────

    pub fn parse(mut self) -> Result<Program, ParseError> {
        let start = self.current_span();
        let mut statements = vec![];
        self.skip_newlines();
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }
        Ok(Program { statements, span: start })
    }

    // ── statements ────────────────────────────────────────

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.current() {
            Token::KwExternal => self.parse_external_entity(),
            Token::KwEntity   => self.parse_entity(),
            Token::KwLet      => self.parse_let(),
            Token::KwRule     => self.parse_rule(),
            Token::KwShow | Token::KwUpdate
            | Token::KwCreate | Token::KwDelete => {
                Ok(Statement::Action(self.parse_action()?))
            }
            _ => Err(ParseError::new(
                format!("unexpected token: {:?}", self.current()),
                self.current_span(),
            )),
        }
    }

    // ── entity ────────────────────────────────────────────

    fn parse_entity(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.advance(); // consume 'entity'
        let name = self.expect_ident("entity name")?;
        self.expect(&Token::LBrace)?;
        self.skip_newlines();
        let mut fields = vec![];
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            fields.push(self.parse_field()?);
            self.skip_newlines();
        }
        self.expect(&Token::RBrace)?;
        Ok(Statement::Entity(EntityDecl { name, fields, span: start }))
    }

    fn parse_field(&mut self) -> Result<Field, ParseError> {
        let metadata = self.parse_metadata()?;
        let start = self.current_span();
        let name = self.expect_ident("field name")?;

        if self.check(&Token::Colon) {
            self.advance();
            let ty = self.parse_type()?;
            Ok(Field::Stored(StoredField { name, ty, metadata, span: start }))
        } else if self.check(&Token::ColonEq) {
            self.advance();
            let expr = self.parse_expr(0)?;
            Ok(Field::Derived(DerivedField { name, expr, metadata, span: start }))
        } else {
            Err(ParseError::new("expected ':' or ':=' after field name", self.current_span()))
        }
    }

    fn parse_metadata(&mut self) -> Result<FieldMetadata, ParseError> {
        let mut meta = FieldMetadata::default();
        while self.check(&Token::At) {
            self.advance(); // '@'
            let tag = self.expect_ident("metadata tag")?;
            match tag.as_str() {
                "doc" => {
                    meta.doc = Some(self.expect_text("@doc string")?);
                }
                "range" => {
                    let low = self.expect_number("@range low")?;
                    self.expect(&Token::KwTo)?;
                    let high = self.expect_number("@range high")?;
                    meta.range = Some((low, high));
                }
                "affects" => {
                    meta.affects.push(self.expect_ident("@affects field")?);
                    while self.check(&Token::Comma) {
                        self.advance();
                        meta.affects.push(self.expect_ident("@affects field")?);
                    }
                }
                _ => return Err(ParseError::new(
                    format!("unknown metadata tag '@{}'", tag),
                    self.current_span(),
                )),
            }
            self.skip_newlines();
        }
        Ok(meta)
    }

    fn parse_type(&mut self) -> Result<LuminaType, ParseError> {
        let t = match self.current() {
            Token::KwTypeText    => { self.advance(); LuminaType::Text }
            Token::KwTypeNumber  => { self.advance(); LuminaType::Number }
            Token::KwTypeBoolean => { self.advance(); LuminaType::Boolean }
            Token::Ident(_) => {
                let name = self.expect_ident("type name")?;
                LuminaType::Entity(name)
            }
            _ => return Err(ParseError::new("expected type", self.current_span())),
        };
        Ok(t)
    }

    // ── external entity ───────────────────────────────────

    fn parse_external_entity(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.advance(); // 'external'
        self.expect(&Token::KwEntity)?;
        let name = self.expect_ident("entity name")?;
        self.expect(&Token::LBrace)?;
        self.skip_newlines();

        let mut fields = vec![];
        let mut sync_path = String::new();
        let mut sync_strategy = SyncStrategy::Realtime;
        let poll_interval = None;

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if self.check(&Token::KwSync) {
                self.advance();
                self.expect(&Token::Colon)?;
                sync_path = self.expect_text("sync path")?;
            } else if self.check(&Token::KwOn) {
                self.advance();
                self.expect(&Token::Colon)?;
                let strategy_str = self.expect_text("sync strategy")?;
                sync_strategy = match strategy_str.as_str() {
                    "realtime" => SyncStrategy::Realtime,
                    "poll"     => SyncStrategy::Poll,
                    "webhook"  => SyncStrategy::Webhook,
                    _ => return Err(ParseError::new(
                        format!("unknown sync strategy '{}'", strategy_str),
                        self.current_span(),
                    )),
                };
            } else {
                fields.push(self.parse_field()?);
            }
            self.skip_newlines();
        }
        self.expect(&Token::RBrace)?;
        Ok(Statement::ExternalEntity(ExternalEntityDecl {
            name, fields, sync_path, sync_strategy, poll_interval, span: start,
        }))
    }

    // ── let ───────────────────────────────────────────────

    fn parse_let(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.advance(); // 'let'
        let name = self.expect_ident("variable name")?;
        self.expect(&Token::Eq)?;

        // Peek: if Ident followed by '{', it's an EntityInit
        let value = if matches!(self.current(), Token::Ident(_))
            && self.peek() == Some(&Token::LBrace)
        {
            let entity_name = self.expect_ident("entity name")?;
            let init = self.parse_entity_init(entity_name)?;
            LetValue::EntityInit(init)
        } else {
            LetValue::Expr(self.parse_expr(0)?)
        };
        Ok(Statement::Let(LetStmt { name, value, span: start }))
    }

    fn parse_entity_init(&mut self, entity_name: String) -> Result<EntityInit, ParseError> {
        let start = self.current_span();
        self.expect(&Token::LBrace)?;
        self.skip_newlines();
        let mut fields = vec![];
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let fname = self.expect_ident("field name")?;
            self.expect(&Token::Colon)?;
            let expr = self.parse_expr(0)?;
            fields.push((fname, expr));
            if self.check(&Token::Comma) { self.advance(); }
            self.skip_newlines();
        }
        self.expect(&Token::RBrace)?;
        Ok(EntityInit { entity_name, fields, span: start })
    }

    // ── rule ──────────────────────────────────────────────

    fn parse_rule(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_span();
        self.advance(); // 'rule'
        let name = self.expect_text("rule name")?;
        self.expect(&Token::LBrace)?;
        self.skip_newlines();

        let trigger = if self.check(&Token::KwWhen) {
            self.advance();
            RuleTrigger::When(self.parse_condition()?)
        } else if self.check(&Token::KwEvery) {
            self.advance();
            RuleTrigger::Every(self.parse_duration()?)
        } else {
            return Err(ParseError::new(
                "expected 'when' or 'every' in rule body",
                self.current_span(),
            ));
        };

        let actions = self.parse_actions()?;
        self.expect(&Token::RBrace)?;
        Ok(Statement::Rule(RuleDecl { name, trigger, actions, span: start }))
    }

    fn parse_condition(&mut self) -> Result<Condition, ParseError> {
        let expr = self.parse_expr(0)?;
        let becomes = if self.check(&Token::KwBecomes) {
            self.advance();
            Some(self.parse_expr(0)?)
        } else { None };
        let for_duration = if self.check(&Token::KwFor) {
            self.advance();
            Some(self.parse_duration()?)
        } else { None };
        Ok(Condition { expr, becomes, for_duration })
    }

    fn parse_duration(&mut self) -> Result<Duration, ParseError> {
        let value = self.expect_number("duration value")?;
        let unit_str = self.expect_ident("time unit (s/m/h/d)")?;
        let unit = match unit_str.as_str() {
            "s" => TimeUnit::Seconds,
            "m" => TimeUnit::Minutes,
            "h" => TimeUnit::Hours,
            "d" => TimeUnit::Days,
            _   => return Err(ParseError::new(
                format!("unknown time unit '{}', expected s/m/h/d", unit_str),
                self.current_span(),
            )),
        };
        Ok(Duration { value, unit })
    }

    fn parse_actions(&mut self) -> Result<Vec<Action>, ParseError> {
        let mut actions = vec![];
        self.skip_newlines();
        while self.check(&Token::KwThen) {
            self.advance();
            actions.push(self.parse_action()?);
            self.skip_newlines();
        }
        if actions.is_empty() {
            return Err(ParseError::new("expected at least one 'then' action", self.current_span()));
        }
        Ok(actions)
    }

    fn parse_action(&mut self) -> Result<Action, ParseError> {
        match self.current() {
            Token::KwShow => {
                self.advance();
                Ok(Action::Show(self.parse_expr(0)?))
            }
            Token::KwUpdate => {
                self.advance();
                let instance = self.expect_ident("instance name")?;
                self.expect(&Token::Dot)?;
                let field = self.expect_ident("field name")?;
                let span = self.current_span();
                self.expect(&Token::KwTo)?;
                let value = self.parse_expr(0)?;
                Ok(Action::Update {
                    target: FieldPath { instance, field, span },
                    value,
                })
            }
            Token::KwCreate => {
                self.advance();
                let entity = self.expect_ident("entity name")?;
                self.expect(&Token::LBrace)?;
                self.skip_newlines();
                let mut fields = vec![];
                while !self.check(&Token::RBrace) && !self.is_at_end() {
                    let fname = self.expect_ident("field name")?;
                    self.expect(&Token::Colon)?;
                    let expr = self.parse_expr(0)?;
                    fields.push((fname, expr));
                    if self.check(&Token::Comma) { self.advance(); }
                    self.skip_newlines();
                }
                self.expect(&Token::RBrace)?;
                Ok(Action::Create { entity, fields })
            }
            Token::KwDelete => {
                self.advance();
                let name = self.expect_ident("instance name")?;
                Ok(Action::Delete(name))
            }
            _ => Err(ParseError::new(
                format!("expected action keyword, got {:?}", self.current()),
                self.current_span(),
            )),
        }
    }

    // ── expressions (Pratt) ───────────────────────────────

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_prefix()?;

        while !self.is_at_end() {
            let token = self.current().clone();
            if let Some((l_bp, r_bp)) = infix_bp(&token) {
                if l_bp < min_bp { break; }
                if token == Token::Dot {
                    self.advance();
                    let field = self.expect_ident("field name")?;
                    let span = self.current_span();
                    lhs = Expr::FieldAccess {
                        obj: Box::new(lhs), field, span,
                    };
                } else {
                    self.advance();
                    let rhs = self.parse_expr(r_bp)?;
                    lhs = Expr::Binary {
                        op: token_to_binop(&token),
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                        span: Span::default(),
                    };
                }
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::new("unexpected end of input in expression", self.current_span()));
        }
        let span = self.current_span();
        let tok = self.current().clone();
        match tok {
            Token::Number(n) => { self.advance(); Ok(Expr::Number(n)) }
            Token::Text(ref s) => {
                let s = s.clone();
                self.advance();
                self.parse_text_or_interpolated(&s, span)
            }
            Token::KwTrue  => { self.advance(); Ok(Expr::Bool(true)) }
            Token::KwFalse => { self.advance(); Ok(Expr::Bool(false)) }
            Token::Ident(ref name) => {
                let name = name.clone();
                self.advance();
                Ok(Expr::Ident(name))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr(0)?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Token::Minus => {
                self.advance();
                let bp = prefix_bp(&Token::Minus).unwrap();
                let operand = self.parse_expr(bp)?;
                Ok(Expr::Unary { op: UnOp::Neg, operand: Box::new(operand), span })
            }
            Token::KwNot => {
                self.advance();
                let bp = prefix_bp(&Token::KwNot).unwrap();
                let operand = self.parse_expr(bp)?;
                Ok(Expr::Unary { op: UnOp::Not, operand: Box::new(operand), span })
            }
            Token::KwIf => {
                self.advance();
                let cond = self.parse_expr(0)?;
                self.expect(&Token::KwThen)?;
                let then_ = self.parse_expr(0)?;
                self.expect(&Token::KwElse)?;
                let else_ = self.parse_expr(0)?;
                Ok(Expr::If {
                    cond: Box::new(cond),
                    then_: Box::new(then_),
                    else_: Box::new(else_),
                    span,
                })
            }
            _ => Err(ParseError::new(
                format!("unexpected token in expression: {:?}", tok),
                span,
            )),
        }
    }

    // ── text interpolation ────────────────────────────────

    fn parse_text_or_interpolated(&self, s: &str, span: Span) -> Result<Expr, ParseError> {
        if !s.contains('{') {
            return Ok(Expr::Text(s.to_string()));
        }
        let mut segments = vec![];
        let mut literal = String::new();
        let mut chars = s.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                if !literal.is_empty() {
                    segments.push(Segment::Literal(std::mem::take(&mut literal)));
                }
                let mut expr_str = String::new();
                let mut closed = false;
                for c in chars.by_ref() {
                    if c == '}' { closed = true; break; }
                    expr_str.push(c);
                }
                if !closed {
                    return Err(ParseError::new(
                        "unclosed string interpolation '{', expected '}'".to_string(),
                        span,
                    ));
                }
                let trimmed = expr_str.trim();
                if trimmed.contains('.') {
                    let parts: Vec<&str> = trimmed.split('.').collect();
                    let mut expr = Expr::Ident(parts[0].to_string());
                    for part in &parts[1..] {
                        expr = Expr::FieldAccess {
                            obj: Box::new(expr),
                            field: part.to_string(),
                            span: Span::default(),
                        };
                    }
                    segments.push(Segment::Expr(expr));
                } else {
                    segments.push(Segment::Expr(Expr::Ident(trimmed.to_string())));
                }
            } else {
                literal.push(ch);
            }
        }
        if !literal.is_empty() {
            segments.push(Segment::Literal(literal));
        }
        Ok(Expr::Interpolated { segments, span })
    }

    // ── convenience extractors ────────────────────────────

    fn expect_ident(&mut self, ctx: &str) -> Result<String, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::new(format!("expected {} but got end of input", ctx), self.current_span()));
        }
        match self.current().clone() {
            Token::Ident(name) => { self.advance(); Ok(name) }
            other => Err(ParseError::new(
                format!("expected {} (identifier), got {:?}", ctx, other),
                self.current_span(),
            )),
        }
    }

    fn expect_text(&mut self, ctx: &str) -> Result<String, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::new(format!("expected {} but got end of input", ctx), self.current_span()));
        }
        match self.current().clone() {
            Token::Text(s) => { self.advance(); Ok(s) }
            other => Err(ParseError::new(
                format!("expected {} (string), got {:?}", ctx, other),
                self.current_span(),
            )),
        }
    }

    fn expect_number(&mut self, ctx: &str) -> Result<f64, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::new(format!("expected {} but got end of input", ctx), self.current_span()));
        }
        match self.current().clone() {
            Token::Number(n) => { self.advance(); Ok(n) }
            other => Err(ParseError::new(
                format!("expected {} (number), got {:?}", ctx, other),
                self.current_span(),
            )),
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use lumina_lexer::tokenize;
    use super::*;

    fn parse_source(source: &str) -> Program {
        let tokens = tokenize(source).expect("lexing failed");
        Parser::new(tokens).parse().expect("parsing failed")
    }

    #[test]
    fn test_entity_with_stored_and_derived_fields() {
        let prog = parse_source(
            "entity Person {\n  name: Text\n  age: Number\n  isAdult := age >= 18\n}"
        );
        assert_eq!(prog.statements.len(), 1);
        match &prog.statements[0] {
            Statement::Entity(e) => {
                assert_eq!(e.name, "Person");
                assert_eq!(e.fields.len(), 3);
                assert!(matches!(&e.fields[0], Field::Stored(f) if f.name == "name"));
                assert!(matches!(&e.fields[1], Field::Stored(f) if f.name == "age"));
                assert!(matches!(&e.fields[2], Field::Derived(f) if f.name == "isAdult"));
            }
            _ => panic!("expected Entity statement"),
        }
    }

    #[test]
    fn test_rule_with_becomes_and_for_duration() {
        let prog = parse_source(concat!(
            "rule \"lock bike\" {\n",
            "  when isIdle becomes true for 10 m\n",
            "  then show \"Bike locked\"\n",
            "}"
        ));
        assert_eq!(prog.statements.len(), 1);
        match &prog.statements[0] {
            Statement::Rule(r) => {
                assert_eq!(r.name, "lock bike");
                match &r.trigger {
                    RuleTrigger::When(c) => {
                        assert!(c.becomes.is_some());
                        assert!(c.for_duration.is_some());
                        let d = c.for_duration.as_ref().unwrap();
                        assert_eq!(d.value, 10.0);
                        assert!(matches!(d.unit, TimeUnit::Minutes));
                    }
                    _ => panic!("expected When trigger"),
                }
                assert_eq!(r.actions.len(), 1);
            }
            _ => panic!("expected Rule statement"),
        }
    }

    #[test]
    fn test_rule_with_every() {
        let prog = parse_source(concat!(
            "rule \"hourly check\" {\n",
            "  every 1 h\n",
            "  then show \"check\"\n",
            "}"
        ));
        match &prog.statements[0] {
            Statement::Rule(r) => {
                assert_eq!(r.name, "hourly check");
                match &r.trigger {
                    RuleTrigger::Every(d) => {
                        assert_eq!(d.value, 1.0);
                        assert!(matches!(d.unit, TimeUnit::Hours));
                    }
                    _ => panic!("expected Every trigger"),
                }
            }
            _ => panic!("expected Rule statement"),
        }
    }

    #[test]
    fn test_let_with_entity_init() {
        let prog = parse_source(
            "let isaac = Person { name: \"Isaac\", birthYear: 2000 }"
        );
        match &prog.statements[0] {
            Statement::Let(l) => {
                assert_eq!(l.name, "isaac");
                match &l.value {
                    LetValue::EntityInit(init) => {
                        assert_eq!(init.entity_name, "Person");
                        assert_eq!(init.fields.len(), 2);
                        assert_eq!(init.fields[0].0, "name");
                        assert_eq!(init.fields[1].0, "birthYear");
                    }
                    _ => panic!("expected EntityInit"),
                }
            }
            _ => panic!("expected Let statement"),
        }
    }

    #[test]
    fn test_if_then_else_expression() {
        let prog = parse_source(
            "let status = if age >= 18 then \"adult\" else \"minor\""
        );
        match &prog.statements[0] {
            Statement::Let(l) => {
                assert_eq!(l.name, "status");
                match &l.value {
                    LetValue::Expr(Expr::If { cond, then_, else_, .. }) => {
                        assert!(matches!(cond.as_ref(), Expr::Binary { op: BinOp::Ge, .. }));
                        assert!(matches!(then_.as_ref(), Expr::Text(s) if s == "adult"));
                        assert!(matches!(else_.as_ref(), Expr::Text(s) if s == "minor"));
                    }
                    _ => panic!("expected If expression"),
                }
            }
            _ => panic!("expected Let statement"),
        }
    }

    #[test]
    fn test_multi_action_rule() {
        let prog = parse_source(concat!(
            "rule \"multi\" {\n",
            "  when score >= 100\n",
            "  then show \"Winner!\"\n",
            "  then update player.status to \"champion\"\n",
            "}"
        ));
        match &prog.statements[0] {
            Statement::Rule(r) => {
                assert_eq!(r.name, "multi");
                assert_eq!(r.actions.len(), 2);
                assert!(matches!(&r.actions[0], Action::Show(_)));
                assert!(matches!(&r.actions[1], Action::Update { .. }));
            }
            _ => panic!("expected Rule statement"),
        }
    }
}
