use tower_lsp::lsp_types::*;
use lumina_lexer::token::Span;
use lumina_parser::ast::*;

/// Walk the AST and collect all locations where `target_name` appears as a symbol.
/// Returns (uri, locations) pairs. The caller provides the document URI.
pub fn find_references_in_program(prog: &Program, target_name: &str, uri: &Url) -> Vec<Location> {
    let mut locs = Vec::new();

    for stmt in &prog.statements {
        collect_from_statement(stmt, target_name, uri, &mut locs);
    }

    locs
}

/// Resolve what symbol is at a given cursor position.
/// Returns the symbol name if found.
pub fn symbol_at_position(prog: &Program, src: &str, pos: Position) -> Option<String> {
    let line = pos.line;
    let col = pos.character;

    for stmt in &prog.statements {
        match stmt {
            Statement::Entity(e) => {
                if span_contains(&e.span, line, col) {
                    // Check if cursor is on the entity name
                    let _name_start = e.span.col.saturating_sub(1);
                    // "entity " is 7 chars, but the span start is at "entity" keyword
                    // The name would be after "entity " — approximate by checking name vicinity
                    if let Some(name) = check_name_at(src, &e.name, line, col) {
                        return Some(name);
                    }
                }
                for f in &e.fields {
                    if let Some(name) = symbol_in_field(f, src, line, col) {
                        return Some(name);
                    }
                }
            }
            Statement::ExternalEntity(e) => {
                if let Some(name) = check_name_at(src, &e.name, line, col) {
                    return Some(name);
                }
                for f in &e.fields {
                    if let Some(name) = symbol_in_field(f, src, line, col) {
                        return Some(name);
                    }
                }
            }
            Statement::Rule(r) => {
                if let Some(name) = check_name_at(src, &r.name, line, col) {
                    return Some(name);
                }
                // Check trigger expressions
                if let Some(name) = symbol_in_trigger(&r.trigger, src, line, col) {
                    return Some(name);
                }
            }
            Statement::Let(l) => {
                if let Some(name) = check_name_at(src, &l.name, line, col) {
                    return Some(name);
                }
            }
            Statement::Fn(f) => {
                if let Some(name) = check_name_at(src, &f.name, line, col) {
                    return Some(name);
                }
            }
            Statement::Aggregate(a) => {
                if let Some(name) = check_name_at(src, &a.name, line, col) {
                    return Some(name);
                }
            }
            _ => {}
        }
    }
    None
}

/// Build rename edits: find all occurrences of `old_name` and replace with `new_name`.
pub fn build_rename_edits(prog: &Program, _src: &str, uri: &Url, old_name: &str, new_name: &str) -> Vec<TextEdit> {
    let refs = find_references_in_program(prog, old_name, uri);
    refs.into_iter()
        .map(|loc| TextEdit {
            range: loc.range,
            new_text: new_name.to_string(),
        })
        .collect()
}

// ── Internal helpers ───────────────────────────────────────────────────────

fn collect_from_statement(stmt: &Statement, name: &str, uri: &Url, locs: &mut Vec<Location>) {
    match stmt {
        Statement::Entity(e) => {
            if e.name == name { locs.push(span_to_location(&e.span, &e.name, uri)); }
            collect_from_fields(&e.fields, name, uri, locs);
        }
        Statement::ExternalEntity(e) => {
            if e.name == name { locs.push(span_to_location(&e.span, &e.name, uri)); }
            collect_from_fields(&e.fields, name, uri, locs);
        }
        Statement::Rule(r) => {
            if r.name == name { locs.push(span_to_location(&r.span, &r.name, uri)); }
            collect_from_trigger(&r.trigger, name, uri, locs);
            for action in &r.actions {
                collect_from_action(action, name, uri, locs);
            }
            if let Some(on_clear) = &r.on_clear {
                for action in on_clear {
                    collect_from_action(action, name, uri, locs);
                }
            }
        }
        Statement::Let(l) => {
            if l.name == name { locs.push(span_to_location(&l.span, &l.name, uri)); }
            match &l.value {
                LetValue::Expr(e) => collect_from_expr(e, name, uri, locs),
                LetValue::EntityInit(init) => {
                    if init.entity_name == name { locs.push(span_to_location(&init.span, &init.entity_name, uri)); }
                    for (_, expr) in &init.fields {
                        collect_from_expr(expr, name, uri, locs);
                    }
                }
            }
        }
        Statement::Fn(f) => {
            if f.name == name { locs.push(span_to_location(&f.span, &f.name, uri)); }
            collect_from_expr(&f.body, name, uri, locs);
        }
        Statement::Aggregate(a) => {
            if a.name == name { locs.push(span_to_location(&a.span, &a.name, uri)); }
            if a.over == name { locs.push(span_to_location(&a.span, &a.over, uri)); }
        }
        Statement::Import(_) | Statement::Action(_) => {}
    }
}

fn collect_from_fields(fields: &[Field], name: &str, uri: &Url, locs: &mut Vec<Location>) {
    for f in fields {
        match f {
            Field::Stored(sf) => {
                if sf.name == name { locs.push(span_to_location(&sf.span, &sf.name, uri)); }
            }
            Field::Derived(df) => {
                if df.name == name { locs.push(span_to_location(&df.span, &df.name, uri)); }
                collect_from_expr(&df.expr, name, uri, locs);
            }
            Field::Ref(rf) => {
                if rf.name == name { locs.push(span_to_location(&rf.span, &rf.name, uri)); }
                if rf.target_entity == name { locs.push(span_to_location(&rf.span, &rf.target_entity, uri)); }
            }
        }
    }
}

fn collect_from_trigger(trigger: &RuleTrigger, name: &str, uri: &Url, locs: &mut Vec<Location>) {
    match trigger {
        RuleTrigger::When(conditions) => {
            for cond in conditions {
                collect_from_expr(&cond.expr, name, uri, locs);
                if let Some(b) = &cond.becomes { collect_from_expr(b, name, uri, locs); }
            }
        }
        RuleTrigger::Any(fc) | RuleTrigger::All(fc) => {
            if fc.entity == name || fc.field == name {}
            collect_from_expr(&fc.becomes, name, uri, locs);
        }
        RuleTrigger::Every(_) => {}
    }
}

fn collect_from_action(action: &Action, name: &str, uri: &Url, locs: &mut Vec<Location>) {
    match action {
        Action::Show(e) => collect_from_expr(e, name, uri, locs),
        Action::Update { target, value } | Action::Write { target, value } => {
            if target.instance == name { locs.push(span_to_location(&target.span, &target.instance, uri)); }
            collect_from_expr(value, name, uri, locs);
        }
        Action::Create { entity: _, fields } => {
            // Can't easily get span for entity name in Create action without AST changes
            for (_, expr) in fields {
                collect_from_expr(expr, name, uri, locs);
            }
        }
        Action::Delete(_) => {}
        Action::Alert(a) => {
            collect_from_expr(&a.severity, name, uri, locs);
            collect_from_expr(&a.message, name, uri, locs);
            if let Some(s) = &a.source { collect_from_expr(s, name, uri, locs); }
        }
    }
}

fn collect_from_expr(expr: &Expr, name: &str, uri: &Url, locs: &mut Vec<Location>) {
    match expr {
        Expr::Ident(_id) => {
            // Ident doesn't carry a span, but we know it references this symbol
            // Without a span on Ident we can't add a precise location
        }
        Expr::FieldAccess { obj, field, span } => {
            if field == name { locs.push(span_to_location(span, name, uri)); }
            collect_from_expr(obj, name, uri, locs);
        }
        Expr::Binary { left, right, .. } => {
            collect_from_expr(left, name, uri, locs);
            collect_from_expr(right, name, uri, locs);
        }
        Expr::Unary { operand, .. } => collect_from_expr(operand, name, uri, locs),
        Expr::If { cond, then_, else_, .. } => {
            collect_from_expr(cond, name, uri, locs);
            collect_from_expr(then_, name, uri, locs);
            collect_from_expr(else_, name, uri, locs);
        }
        Expr::Call { name: fn_name, args, span } => {
            if fn_name == name { locs.push(span_to_location(span, name, uri)); }
            for a in args { collect_from_expr(a, name, uri, locs); }
        }
        Expr::ListLiteral(items) => {
            for item in items { collect_from_expr(item, name, uri, locs); }
        }
        Expr::Index { list, index, .. } => {
            collect_from_expr(list, name, uri, locs);
            collect_from_expr(index, name, uri, locs);
        }
        Expr::InterpolatedString(segments) => {
            for seg in segments {
                if let StringSegment::Expr(e) = seg {
                    collect_from_expr(e, name, uri, locs);
                }
            }
        }
        Expr::Prev { .. } | Expr::Number(_) | Expr::Text(_) | Expr::Bool(_) => {}
    }
}

fn span_to_location(span: &Span, name: &str, uri: &Url) -> Location {
    let l = span.line.saturating_sub(1);
    let c = span.col.saturating_sub(1);
    Location {
        uri: uri.clone(),
        range: Range {
            start: Position { line: l, character: c },
            end: Position { line: l, character: c + name.len() as u32 },
        },
    }
}

fn span_contains(span: &Span, line: u32, _col: u32) -> bool {
    span.line.saturating_sub(1) == line
}

fn check_name_at(src: &str, name: &str, line: u32, col: u32) -> Option<String> {
    // Find the word at (line, col) in the source and check if it matches
    let target_line = src.lines().nth(line as usize)?;
    let col = col as usize;
    if col >= target_line.len() { return None; }

    // Find word boundaries around the cursor
    let start = target_line[..col].rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1).unwrap_or(0);
    let end = target_line[col..].find(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| col + i).unwrap_or(target_line.len());

    let word = &target_line[start..end];
    if word == name { Some(name.to_string()) } else { None }
}

fn symbol_in_field(f: &Field, src: &str, line: u32, col: u32) -> Option<String> {
    match f {
        Field::Stored(sf) => check_name_at(src, &sf.name, line, col),
        Field::Derived(df) => check_name_at(src, &df.name, line, col),
        Field::Ref(rf) => {
            check_name_at(src, &rf.name, line, col)
                .or_else(|| check_name_at(src, &rf.target_entity, line, col))
        }
    }
}

fn symbol_in_trigger(trigger: &RuleTrigger, src: &str, line: u32, col: u32) -> Option<String> {
    match trigger {
        RuleTrigger::When(conditions) => {
            for c in conditions {
                if let Some(name) = symbol_in_expr(&c.expr, src, line, col) {
                    return Some(name);
                }
            }
            None
        }
        RuleTrigger::Any(fc) | RuleTrigger::All(fc) => {
            check_name_at(src, &fc.entity, line, col)
                .or_else(|| check_name_at(src, &fc.field, line, col))
        }
        RuleTrigger::Every(_) => None,
    }
}

fn symbol_in_expr(expr: &Expr, src: &str, line: u32, col: u32) -> Option<String> {
    match expr {
        Expr::FieldAccess { obj, field, span } => {
            let sl = span.line.saturating_sub(1);
            if sl == line {
                if let Some(name) = check_name_at(src, field, line, col) {
                    return Some(name);
                }
            }
            symbol_in_expr(obj, src, line, col)
        }
        Expr::Call { name, args, span } => {
            let sl = span.line.saturating_sub(1);
            if sl == line {
                if let Some(n) = check_name_at(src, name, line, col) {
                    return Some(n);
                }
            }
            for a in args {
                if let Some(n) = symbol_in_expr(a, src, line, col) {
                    return Some(n);
                }
            }
            None
        }
        Expr::Binary { left, right, .. } => {
            symbol_in_expr(left, src, line, col)
                .or_else(|| symbol_in_expr(right, src, line, col))
        }
        Expr::Unary { operand, .. } => symbol_in_expr(operand, src, line, col),
        Expr::If { cond, then_, else_, .. } => {
            symbol_in_expr(cond, src, line, col)
                .or_else(|| symbol_in_expr(then_, src, line, col))
                .or_else(|| symbol_in_expr(else_, src, line, col))
        }
        Expr::Ident(name) => {
            // Ident has no span, but we can try the name check
            check_name_at(src, name, line, col)
        }
        _ => None,
    }
}
