use std::collections::HashMap;
use lumina_parser::ast::*;
use lumina_lexer::token::Span;
use crate::types::{Schema, EntitySchema, FieldSchema};
use crate::graph::DependencyGraph;

#[derive(Debug)]
pub struct AnalyzerError {
    pub code:    &'static str,
    pub message: String,
    pub span:    Span,
}

/// The output of a successful analysis pass
#[derive(Debug)]
pub struct AnalyzedProgram {
    pub program: Program,
    pub schema:  Schema,
    pub graph:   DependencyGraph,
    pub fn_defs: HashMap<String, FnDecl>,
    pub instances: HashMap<String, LuminaType>,
}

pub struct Analyzer {
    schema: Schema,
    graph:  DependencyGraph,
    pub errors: Vec<AnalyzerError>,
    pub allow_imports: bool,
    locals: HashMap<String, LuminaType>,
    pub fn_defs: HashMap<String, FnDecl>,
    pub instances: HashMap<String, LuminaType>,
    in_prev_context: bool,
    in_derived_context: bool,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            schema: Schema::new(),
            graph: DependencyGraph::new(),
            errors: Vec::new(),
            allow_imports: true,
            locals: HashMap::new(),
            fn_defs: HashMap::new(),
            instances: HashMap::new(),
            in_prev_context: false,
            in_derived_context: false,
        }
    }

    pub fn analyze(mut self, program: Program) -> Result<AnalyzedProgram, Vec<AnalyzerError>> {
        self.pass1_register_entities(&program);
        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        self.pass2_typecheck(&program)?;

        Ok(AnalyzedProgram {
            program,
            schema: self.schema,
            graph: self.graph,
            fn_defs: self.fn_defs,
            instances: self.instances,
        })
    }

    fn pass1_register_entities(&mut self, program: &Program) {
        for stmt in &program.statements {
            match stmt {
                Statement::Entity(decl) => self.register_entity(decl, false),
                Statement::ExternalEntity(decl) => self.register_external_entity(decl),
                Statement::Fn(decl) => {
                    if self.fn_defs.contains_key(&decl.name) {
                        self.errors.push(AnalyzerError {
                            code: "L011",
                            message: format!("duplicate fn name: {}", decl.name),
                            span: decl.span,
                        });
                    } else {
                        self.fn_defs.insert(decl.name.clone(), decl.clone());
                    }
                }
                Statement::Import(decl) => {
                    if !self.allow_imports {
                        self.errors.push(AnalyzerError {
                            code: "L018",
                            message: "import is not supported in single-file (WASM) mode".to_string(),
                            span: decl.span,
                        });
                    }
                }
                Statement::Let(decl) => {
                    match &decl.value {
                        LetValue::EntityInit(init) => {
                            self.instances.insert(decl.name.clone(), LuminaType::Entity(init.entity_name.clone()));
                        }
                        LetValue::Expr(expr) => {
                             if let Ok(ty) = self.infer_type(expr, None, None) {
                                self.instances.insert(decl.name.clone(), ty);
                             }
                        }
                    }
                }
                Statement::Aggregate(_) => {}
                _ => {}
            }
        }
    }

    fn register_entity(&mut self, decl: &EntityDecl, is_external: bool) {
        if self.schema.entities.contains_key(&decl.name) {
            self.errors.push(AnalyzerError {
                code: "L005",
                message: format!("Duplicate entity name: {}", decl.name),
                span: decl.span,
            });
            return;
        }

        let mut fields = HashMap::new();
        for field in &decl.fields {
            let (name, schema_field) = match field {
                Field::Stored(f) => {
                    (f.name.clone(), FieldSchema {
                        name: f.name.clone(),
                        ty: f.ty.clone(),
                        is_derived: false,
                        metadata: f.metadata.clone(),
                    })
                }
                Field::Derived(f) => {
                    (f.name.clone(), FieldSchema {
                        name: f.name.clone(),
                        ty: LuminaType::Number, // Placeholder, resolved in pass 2
                        is_derived: true,
                        metadata: f.metadata.clone(),
                    })
                }
                Field::Ref(r) => {
                    // L036: Validate ref target entity exists
                    if !self.schema.entities.contains_key(&r.target_entity) {
                        // Defer check — target may not be registered yet in pass1
                        // We'll validate in pass2 instead
                    }
                    (r.name.clone(), FieldSchema {
                        name: r.name.clone(),
                        ty: LuminaType::Entity(r.target_entity.clone()),
                        is_derived: false,
                        metadata: FieldMetadata::default(),
                    })
                }
            };

            if fields.contains_key(&name) {
                self.errors.push(AnalyzerError {
                    code: "L006",
                    message: format!("Duplicate field name: {}", name),
                    span: decl.span, // Simplified span for field error
                });
            } else {
                fields.insert(name, schema_field);
            }
        }

        self.schema.entities.insert(decl.name.clone(), EntitySchema {
            name: decl.name.clone(),
            fields,
            is_external,
            sync_on: None,
        });
    }

    fn register_external_entity(&mut self, decl: &ExternalEntityDecl) {
        // Reuse register_entity logic by converting ExternalEntityDecl to EntityDecl structure
        if self.schema.entities.contains_key(&decl.name) {
            self.errors.push(AnalyzerError {
                code: "L005",
                message: format!("Duplicate entity name: {}", decl.name),
                span: decl.span,
            });
            return;
        }

        let mut fields = HashMap::new();
        for field in &decl.fields {
            let (name, schema_field) = match field {
                Field::Stored(f) => {
                    (f.name.clone(), FieldSchema {
                        name: f.name.clone(),
                        ty: f.ty.clone(),
                        is_derived: false,
                        metadata: f.metadata.clone(),
                    })
                }
                Field::Derived(f) => {
                    (f.name.clone(), FieldSchema {
                        name: f.name.clone(),
                        ty: LuminaType::Number,
                        is_derived: true,
                        metadata: f.metadata.clone(),
                    })
                }
                Field::Ref(r) => {
                    (r.name.clone(), FieldSchema {
                        name: r.name.clone(),
                        ty: LuminaType::Entity(r.target_entity.clone()),
                        is_derived: false,
                        metadata: FieldMetadata::default(),
                    })
                }
            };
            fields.insert(name, schema_field);
        }

        self.schema.entities.insert(decl.name.clone(), EntitySchema {
            name: decl.name.clone(),
            fields,
            is_external: true,
            sync_on: if decl.sync_path.is_empty() { None } else { Some(decl.sync_path.clone()) },
        });
    }

    fn pass2_typecheck(&mut self, program: &Program) -> Result<(), Vec<AnalyzerError>> {
        // L036: Validate all ref targets exist now that all entities are registered
        for stmt in &program.statements {
            let fields = match stmt {
                Statement::Entity(e) => Some((&e.fields, e.span)),
                Statement::ExternalEntity(e) => Some((&e.fields, e.span)),
                _ => None,
            };
            if let Some((fields, span)) = fields {
                for field in fields {
                    if let Field::Ref(r) = field {
                        if !self.schema.entities.contains_key(&r.target_entity) {
                            return Err(vec![AnalyzerError {
                                code: "L036",
                                message: format!("ref target entity '{}' does not exist", r.target_entity),
                                span: r.span,
                            }]);
                        }
                    }
                }
            }
        }

        for stmt in &program.statements {
            match stmt {
                Statement::Entity(decl) => {
                    for field in &decl.fields {
                        match field {
                            Field::Derived(df) => {
                                let ty = self.infer_type_ctx(&df.expr, Some(&decl.name), None, true).map_err(|e| vec![e])?;
                                if let Some(entity) = self.schema.entities.get_mut(&decl.name) {
                                    if let Some(f_schema) = entity.fields.get_mut(&df.name) {
                                        f_schema.ty = ty;
                                    }
                                }
                                // Build dependency graph for derived fields
                                let target_node = self.graph.intern(&decl.name, &df.name);
                                self.collect_dependencies(&df.expr, &decl.name, target_node)?;
                            }
                            Field::Ref(r) => {
                                // L037: Add ref edges to the dependency graph for cycle detection
                                let ref_node = self.graph.intern(&decl.name, &r.name);
                                let target_node = self.graph.intern(&r.target_entity, "__entity__");
                                self.graph.add_edge(target_node, ref_node);
                            }
                            _ => {}
                        }
                    }
                }
                Statement::Rule(rule) => {
                    // Type check condition
                    match &rule.trigger {
                        RuleTrigger::When(conds) => {
                            // L035: Enforce maximum of 3 AND clauses
                            if conds.len() > 3 {
                                return Err(vec![AnalyzerError {
                                    code: "L035",
                                    message: format!("multi-condition trigger has {} clauses, max is 3", conds.len()),
                                    span: rule.span,
                                }]);
                            }
                            for cond in conds {
                                let ty = self.infer_type(&cond.expr, None, None).map_err(|e| vec![e])?;
                                if ty != LuminaType::Boolean {
                                    return Err(vec![AnalyzerError {
                                        code: "L002",
                                        message: "when condition must be Boolean".to_string(),
                                        span: rule.span,
                                    }]);
                                }
                                if let Some(becomes) = &cond.becomes {
                                    let b_ty = self.infer_type(becomes, None, None).map_err(|e| vec![e])?;
                                    if b_ty != LuminaType::Boolean {
                                        return Err(vec![AnalyzerError {
                                            code: "L002",
                                            message: "becomes condition must be Boolean".to_string(),
                                            span: rule.span,
                                        }]);
                                    }
                                }
                                // L039/L040: Validate frequency conditions
                                if let Some(freq) = &cond.frequency {
                                    if freq.count < 2 {
                                        return Err(vec![AnalyzerError {
                                            code: "L039",
                                            message: format!("frequency count must be >= 2, got {}", freq.count),
                                            span: freq.span,
                                        }]);
                                    }
                                    if freq.within.to_seconds() <= 0.0 {
                                        return Err(vec![AnalyzerError {
                                            code: "L040",
                                            message: "frequency window duration must be > 0".to_string(),
                                            span: freq.span,
                                        }]);
                                    }
                                }
                            }
                        }
                        RuleTrigger::Any(fc) | RuleTrigger::All(fc) => {
                            // L026: entity must exist
                            if let Some(entity_schema) = self.schema.entities.get(&fc.entity) {
                                // L027: field must exist and be Boolean
                                if let Some(field_schema) = entity_schema.fields.get(&fc.field) {
                                    if field_schema.ty != LuminaType::Boolean {
                                        return Err(vec![AnalyzerError {
                                            code: "L027",
                                            message: format!(
                                                "fleet trigger field '{}.{}' must be Boolean, found {:?}",
                                                fc.entity, fc.field, field_schema.ty
                                            ),
                                            span: rule.span,
                                        }]);
                                    }
                                } else {
                                    return Err(vec![AnalyzerError {
                                        code: "L027",
                                        message: format!(
                                            "unknown field '{}' on entity '{}'",
                                            fc.field, fc.entity
                                        ),
                                        span: rule.span,
                                    }]);
                                }
                            } else {
                                return Err(vec![AnalyzerError {
                                    code: "L026",
                                    message: format!("unknown entity '{}' in fleet trigger", fc.entity),
                                    span: rule.span,
                                }]);
                            }
                            // Validate becomes value is Boolean
                            let b_ty = self.infer_type(&fc.becomes, None, None).map_err(|e| vec![e])?;
                            if b_ty != LuminaType::Boolean {
                                return Err(vec![AnalyzerError {
                                    code: "L002",
                                    message: "fleet trigger becomes value must be Boolean".to_string(),
                                    span: rule.span,
                                }]);
                            }
                            // L039/L040: Validate fleet frequency conditions
                            if let Some(freq) = &fc.frequency {
                                if freq.count < 2 {
                                    return Err(vec![AnalyzerError {
                                        code: "L039",
                                        message: format!("frequency count must be >= 2, got {}", freq.count),
                                        span: freq.span,
                                    }]);
                                }
                                if freq.within.to_seconds() <= 0.0 {
                                    return Err(vec![AnalyzerError {
                                        code: "L040",
                                        message: "frequency window duration must be > 0".to_string(),
                                        span: freq.span,
                                    }]);
                                }
                            }
                        }
                        RuleTrigger::Every(_) => {}
                    }

                    // Type check actions
                    for action in &rule.actions {
                        self.check_action(action, rule.span)?;
                    }
                }
                Statement::Fn(decl) => {
                    let mut locals = HashMap::new();
                    let mut locals_set = std::collections::HashSet::new();
                    for param in &decl.params {
                        locals.insert(param.name.clone(), param.type_.clone());
                        locals_set.insert(param.name.clone());
                    }
                    self.check_fn_body(&decl.body, &locals_set, decl.span);

                    if let Ok(body_type) = self.infer_type(&decl.body, None, Some(&locals)) {
                        if body_type != decl.returns {
                            self.errors.push(AnalyzerError {
                                code: "L014",
                                message: "return type mismatch".to_string(),
                                span: decl.span,
                            });
                        }
                    }
                }
                Statement::Aggregate(_) => {}
                _ => {}
            }
        }

        // Check for cycles
        if let Err(err) = self.graph.compute_topo_order() {
            return Err(vec![AnalyzerError {
                code: "L004",
                message: format!("Circular dependency detected: {}", err.chain.join(" -> ")),
                span: program.span,
            }]);
        }

        if !self.errors.is_empty() {
            Err(std::mem::take(&mut self.errors))
        } else {
            Ok(())
        }
    }

    fn check_fn_body(&mut self, expr: &Expr, locals: &std::collections::HashSet<String>, span: Span) {
        match expr {
            Expr::FieldAccess { obj, .. } => {
                if let Expr::Ident(ref name) = **obj {
                    if !locals.contains(name) {
                        self.errors.push(AnalyzerError {
                            code: "L015",
                            message: "fn body cannot access entity fields".to_string(),
                            span,
                        });
                    }
                } else {
                    self.errors.push(AnalyzerError {
                        code: "L015",
                        message: "fn body cannot access entity fields".to_string(),
                        span,
                    });
                }
                self.check_fn_body(obj, locals, span);
            }
            Expr::Binary { left, right, .. } => {
                self.check_fn_body(left, locals, span);
                self.check_fn_body(right, locals, span);
            }
            Expr::Unary { operand, .. } => {
                self.check_fn_body(operand, locals, span);
            }
            Expr::If { cond, then_, else_, .. } => {
                self.check_fn_body(cond, locals, span);
                self.check_fn_body(then_, locals, span);
                self.check_fn_body(else_, locals, span);
            }
            Expr::Call { args, .. } => {
                for arg in args {
                    self.check_fn_body(arg, locals, span);
                }
            }
            Expr::InterpolatedString(segments) => {
                for seg in segments {
                    if let StringSegment::Expr(e) = seg {
                        self.check_fn_body(e, locals, span);
                    }
                }
            }
            Expr::ListLiteral(elems) => {
                for elem in elems {
                    self.check_fn_body(elem, locals, span);
                }
            }
            Expr::Index { list, index, .. } => {
                self.check_fn_body(list, locals, span);
                self.check_fn_body(index, locals, span);
            }
            Expr::Prev { .. } => {}
            _ => {}
        }
    }

    /// Wrapper that sets derived context before delegating to infer_type.
    /// Used for L041: now() is forbidden in derived field expressions.
    fn infer_type_ctx(&mut self, expr: &Expr, entity_ctx: Option<&str>, locals: Option<&HashMap<String, LuminaType>>, is_derived: bool) -> Result<LuminaType, AnalyzerError> {
        let prev = self.in_derived_context;
        self.in_derived_context = is_derived;
        let result = self.infer_type(expr, entity_ctx, locals);
        self.in_derived_context = prev;
        result
    }

    fn infer_type(&self, expr: &Expr, entity_ctx: Option<&str>, locals: Option<&HashMap<String, LuminaType>>) -> Result<LuminaType, AnalyzerError> {
        match expr {
            Expr::Number(_) => Ok(LuminaType::Number),
            Expr::Text(_) | Expr::InterpolatedString(_) => Ok(LuminaType::Text),
            Expr::Bool(_) => Ok(LuminaType::Boolean),
            Expr::Ident(name) => {
                if let Some(locs) = locals {
                    if let Some(ty) = locs.get(name) {
                        return Ok(ty.clone());
                    }
                }
                if let Some(ty) = self.instances.get(name) {
                    return Ok(ty.clone());
                }
                // First check if it's a field in the current entity context
                if let Some(ent) = entity_ctx {
                    if let Some(f) = self.schema.get_field(ent, name) {
                        return Ok(f.ty.clone());
                    }
                }
                // Then check if it's an entity name
                if self.schema.entities.contains_key(name) {
                    Ok(LuminaType::Entity(name.clone()))
                } else {
                    Err(AnalyzerError {
                        code: "L001",
                        message: format!("Unknown identifier: {}", name),
                        span: Span::default(),
                    })
                }
            }
            Expr::FieldAccess { obj, field, span } => {
                let obj_ty = self.infer_type(obj, entity_ctx, locals)?;
                match obj_ty {
                    LuminaType::Entity(e_name) => {
                        if let Some(f) = self.schema.get_field(&e_name, field) {
                            Ok(f.ty.clone())
                        } else {
                            Err(AnalyzerError {
                                code: "L010",
                                message: format!("Unknown field '{}' on entity '{}'", field, e_name),
                                span: *span,
                            })
                        }
                    }
                    // L042: .age on Timestamp returns Number (seconds)
                    LuminaType::Timestamp => {
                        if field == "age" {
                            Ok(LuminaType::Number)
                        } else {
                            Err(AnalyzerError {
                                code: "L042",
                                message: format!("Timestamp only supports '.age' accessor, not '.{}'", field),
                                span: *span,
                            })
                        }
                    }
                    _ => Err(AnalyzerError {
                        code: "L002",
                        message: "Field access only allowed on entities or Timestamps".to_string(),
                        span: *span,
                    }),
                }
            }
            Expr::Binary { op, left, right, span } => {
                let l_ty = self.infer_type(left, entity_ctx, locals)?;
                let r_ty = self.infer_type(right, entity_ctx, locals)?;
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        if l_ty == LuminaType::Number && r_ty == LuminaType::Number {
                            Ok(LuminaType::Number)
                        } else {
                            Err(AnalyzerError {
                                code: "L002",
                                message: "Arithmetic operations require Numbers".to_string(),
                                span: *span,
                            })
                        }
                    }
                    BinOp::Eq | BinOp::Ne | BinOp::Gt | BinOp::Lt | BinOp::Ge | BinOp::Le => {
                        if l_ty == r_ty {
                            Ok(LuminaType::Boolean)
                        } else {
                            Err(AnalyzerError {
                                code: "L002",
                                message: "Comparison requires same types".to_string(),
                                span: *span,
                            })
                        }
                    }
                    BinOp::And | BinOp::Or => {
                        if l_ty == LuminaType::Boolean && r_ty == LuminaType::Boolean {
                            Ok(LuminaType::Boolean)
                        } else {
                            Err(AnalyzerError {
                                code: "L002",
                                message: "Logical operations require Booleans".to_string(),
                                span: *span,
                            })
                        }
                    }
                }
            }
            Expr::Unary { op, operand, span } => {
                let ty = self.infer_type(operand, entity_ctx, locals)?;
                match op {
                    UnOp::Neg => {
                        if ty == LuminaType::Number {
                            Ok(LuminaType::Number)
                        } else {
                            Err(AnalyzerError {
                                code: "L002",
                                message: "Negation requires Number".to_string(),
                                span: *span,
                            })
                        }
                    }
                    UnOp::Not => {
                        if ty == LuminaType::Boolean {
                            Ok(LuminaType::Boolean)
                        } else {
                            Err(AnalyzerError {
                                code: "L002",
                                message: "Logical NOT requires Boolean".to_string(),
                                span: *span,
                            })
                        }
                    }
                }
            }
            Expr::If { cond, then_, else_, span } => {
                let c_ty = self.infer_type(cond, entity_ctx, locals)?;
                if c_ty != LuminaType::Boolean {
                    return Err(AnalyzerError {
                        code: "L002",
                        message: "If condition must be Boolean".to_string(),
                        span: *span,
                    });
                }
                let t_ty = self.infer_type(then_, entity_ctx, locals)?;
                let e_ty = self.infer_type(else_, entity_ctx, locals)?;
                if t_ty == e_ty {
                    Ok(t_ty)
                } else {
                    Err(AnalyzerError {
                        code: "L002",
                        message: "If branches must have same type".to_string(),
                        span: *span,
                    })
                }
            }
            Expr::Call { name, args, span } => {
                // Check built-in functions first
                match name.as_str() {
                    "now" => {
                        if !args.is_empty() {
                            return Err(AnalyzerError { code: "L013", message: "now() takes no arguments".to_string(), span: *span });
                        }
                        // L041: now() cannot be used in derived field expressions
                        if self.in_derived_context {
                            return Err(AnalyzerError {
                                code: "L041",
                                message: "now() cannot be used in derived field expressions".to_string(),
                                span: *span,
                            });
                        }
                        return Ok(LuminaType::Timestamp);
                    }
                    "len" => {
                        if args.len() != 1 {
                            return Err(AnalyzerError { code: "L013", message: format!("len expects 1 arg, got {}", args.len()), span: *span });
                        }
                        let arg_ty = self.infer_type(&args[0], entity_ctx, locals)?;
                        if !matches!(arg_ty, LuminaType::List(_)) {
                            return Err(AnalyzerError { code: "L002", message: "len() requires a list argument".to_string(), span: *span });
                        }
                        return Ok(LuminaType::Number);
                    }
                    "min" | "max" | "sum" => {
                        if args.len() != 1 {
                            return Err(AnalyzerError { code: "L013", message: format!("{} expects 1 arg, got {}", name, args.len()), span: *span });
                        }
                        let arg_ty = self.infer_type(&args[0], entity_ctx, locals)?;
                        if arg_ty != LuminaType::List(Box::new(LuminaType::Number)) {
                            return Err(AnalyzerError { code: "L002", message: format!("{}() requires a Number[] argument", name), span: *span });
                        }
                        return Ok(LuminaType::Number);
                    }
                    "append" => {
                        if args.len() != 2 {
                            return Err(AnalyzerError { code: "L013", message: format!("append expects 2 args, got {}", args.len()), span: *span });
                        }
                        let list_ty = self.infer_type(&args[0], entity_ctx, locals)?;
                        let val_ty = self.infer_type(&args[1], entity_ctx, locals)?;
                        match &list_ty {
                            LuminaType::List(inner) if **inner == val_ty => return Ok(list_ty),
                            LuminaType::List(_) => return Err(AnalyzerError { code: "L002", message: "append value type must match list element type".to_string(), span: *span }),
                            _ => return Err(AnalyzerError { code: "L002", message: "append() first argument must be a list".to_string(), span: *span }),
                        }
                    }
                    "head" => {
                        if args.len() != 1 {
                            return Err(AnalyzerError { code: "L013", message: format!("head expects 1 arg, got {}", args.len()), span: *span });
                        }
                        let arg_ty = self.infer_type(&args[0], entity_ctx, locals)?;
                        match arg_ty {
                            LuminaType::List(inner) => return Ok(*inner),
                            _ => return Err(AnalyzerError { code: "L002", message: "head() requires a list argument".to_string(), span: *span }),
                        }
                    }
                    "tail" => {
                        if args.len() != 1 {
                            return Err(AnalyzerError { code: "L013", message: format!("tail expects 1 arg, got {}", args.len()), span: *span });
                        }
                        let arg_ty = self.infer_type(&args[0], entity_ctx, locals)?;
                        if !matches!(&arg_ty, LuminaType::List(_)) {
                            return Err(AnalyzerError { code: "L002", message: "tail() requires a list argument".to_string(), span: *span });
                        }
                        return Ok(arg_ty);
                    }
                    "at" => {
                        if args.len() != 2 {
                            return Err(AnalyzerError { code: "L013", message: format!("at expects 2 args, got {}", args.len()), span: *span });
                        }
                        let list_ty = self.infer_type(&args[0], entity_ctx, locals)?;
                        let idx_ty = self.infer_type(&args[1], entity_ctx, locals)?;
                        if idx_ty != LuminaType::Number {
                            return Err(AnalyzerError { code: "L002", message: "at() index must be a Number".to_string(), span: *span });
                        }
                        match list_ty {
                            LuminaType::List(inner) => return Ok(*inner),
                            _ => return Err(AnalyzerError { code: "L002", message: "at() first argument must be a list".to_string(), span: *span }),
                        }
                    }
                    _ => {} // Fall through to user-defined fn lookup
                }
                // User-defined function lookup
                let decl = match self.fn_defs.get(name) {
                    Some(d) => d.clone(),
                    None => {
                        return Err(AnalyzerError {
                            code: "L012",
                            message: format!("unknown fn: {}", name),
                            span: *span,
                        });
                    }
                };
                if args.len() != decl.params.len() {
                    return Err(AnalyzerError {
                        code: "L013",
                        message: format!("fn {} expects {} args, got {}", name, decl.params.len(), args.len()),
                        span: *span,
                    });
                }
                for (arg, param) in args.iter().zip(decl.params.iter()) {
                    let arg_ty = self.infer_type(arg, entity_ctx, locals)?;
                    if arg_ty != param.type_ {
                        return Err(AnalyzerError {
                            code: "L013",
                            message: format!("argument type mismatch for parameter {}", param.name),
                            span: *span,
                        });
                    }
                }
                Ok(decl.returns.clone())
            }
            Expr::ListLiteral(elems) => {
                if elems.is_empty() {
                    // Empty list — we can't infer element type, default to Number[]
                    return Ok(LuminaType::List(Box::new(LuminaType::Number)));
                }
                let first_ty = self.infer_type(&elems[0], entity_ctx, locals)?;
                for elem in &elems[1..] {
                    let ty = self.infer_type(elem, entity_ctx, locals)?;
                    if ty != first_ty {
                        return Err(AnalyzerError {
                            code: "L002",
                            message: "all list elements must have the same type".to_string(),
                            span: Span::default(),
                        });
                    }
                }
                Ok(LuminaType::List(Box::new(first_ty)))
            }
            Expr::Index { list, index, span } => {
                let list_ty = self.infer_type(list, entity_ctx, locals)?;
                let idx_ty = self.infer_type(index, entity_ctx, locals)?;
                if idx_ty != LuminaType::Number {
                    return Err(AnalyzerError {
                        code: "L002",
                        message: "list index must be a Number".to_string(),
                        span: *span,
                    });
                }
                match list_ty {
                    LuminaType::List(inner) => Ok(*inner),
                    _ => Err(AnalyzerError {
                        code: "L002",
                        message: "index access only allowed on lists".to_string(),
                        span: *span,
                    }),
                }
            }
            Expr::Prev { field, span } => {
                if self.in_prev_context {
                    return Err(AnalyzerError {
                        code: "L025",
                        message: "Nested prev() expressions are not allowed".to_string(),
                        span: *span,
                    });
                }
                
                let entity_name = entity_ctx.ok_or_else(|| AnalyzerError {
                    code: "L001",
                    message: "prev() can only be used within an entity context".to_string(),
                    span: *span,
                })?;
                
                let field_schema = self.schema.get_field(entity_name, field).ok_or_else(|| AnalyzerError {
                    code: "L010",
                    message: format!("Unknown field '{}' for prev()", field),
                    span: *span,
                })?;
                
                if field_schema.is_derived {
                    return Err(AnalyzerError {
                        code: "L024",
                        message: "prev() cannot be used on derived fields".to_string(),
                        span: *span,
                    });
                }
                
                Ok(field_schema.ty.clone())
            }
        }
    }

    fn collect_dependencies(&mut self, expr: &Expr, entity_name: &str, target_id: u32) -> Result<(), Vec<AnalyzerError>> {
        match expr {
            Expr::Ident(name) => {
                // If it's a field in the same entity
                if self.schema.get_field(entity_name, name).is_some() {
                    let dep_id = self.graph.intern(entity_name, name);
                    self.graph.add_edge(dep_id, target_id);
                }
            }
            Expr::FieldAccess { obj, field, .. } => {
                let obj_ty = self.infer_type(obj, Some(entity_name), None).map_err(|e| vec![e])?;
                if let LuminaType::Entity(e_name) = obj_ty {
                    let dep_id = self.graph.intern(&e_name, field);
                    self.graph.add_edge(dep_id, target_id);
                }
                self.collect_dependencies(obj, entity_name, target_id)?;
            }
            Expr::Binary { left, right, .. } => {
                self.collect_dependencies(left, entity_name, target_id)?;
                self.collect_dependencies(right, entity_name, target_id)?;
            }
            Expr::Unary { operand, .. } => {
                self.collect_dependencies(operand, entity_name, target_id)?;
            }
            Expr::If { cond, then_, else_, .. } => {
                self.collect_dependencies(cond, entity_name, target_id)?;
                self.collect_dependencies(then_, entity_name, target_id)?;
                self.collect_dependencies(else_, entity_name, target_id)?;
            }
            Expr::InterpolatedString(segments) => {
                for seg in segments {
                    if let StringSegment::Expr(e) = seg {
                        self.collect_dependencies(e, entity_name, target_id)?;
                    }
                }
            }
            Expr::Call { args, .. } => {
                for arg in args {
                    self.collect_dependencies(arg, entity_name, target_id)?;
                }
            }
            Expr::ListLiteral(elems) => {
                for elem in elems {
                    self.collect_dependencies(elem, entity_name, target_id)?;
                }
            }
            Expr::Index { list, index, .. } => {
                self.collect_dependencies(list, entity_name, target_id)?;
                self.collect_dependencies(index, entity_name, target_id)?;
            }
            Expr::Prev { field, .. } => {
                if self.schema.get_field(entity_name, field).is_some() {
                    let dep_id = self.graph.intern(entity_name, field);
                    self.graph.add_edge(dep_id, target_id);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn check_action(&mut self, action: &Action, rule_span: Span) -> Result<(), Vec<AnalyzerError>> {
        match action {
            Action::Show(expr) => {
                self.infer_type(expr, None, None).map_err(|e| vec![e])?;
                Ok(())
            }
            Action::Update { target, value } => {
                let entity_name = match self.instances.get(&target.instance) {
                    Some(LuminaType::Entity(e)) => e,
                    _ => &target.instance,
                };
                let field_schema = self.schema.get_field(entity_name, &target.field).ok_or_else(|| vec![AnalyzerError {
                    code: "L010",
                    message: format!("Unknown field '{}' on entity '{}'", target.field, target.instance),
                    span: target.span,
                }])?;

                if field_schema.is_derived {
                    return Err(vec![AnalyzerError {
                        code: "L003",
                        message: "Cannot update a derived field".to_string(),
                        span: target.span,
                    }]);
                }

                let val_ty = self.infer_type(value, None, None).map_err(|e| vec![e])?;
                if val_ty != field_schema.ty {
                    return Err(vec![AnalyzerError {
                        code: "L002",
                        message: "Type mismatch in update".to_string(),
                        span: target.span,
                    }]);
                }
                Ok(())
            }
            Action::Create { entity, fields } => {
                let schema_entity = self.schema.get_entity(entity).ok_or_else(|| vec![AnalyzerError {
                    code: "L008",
                    message: format!("Unknown entity type: {}", entity),
                    span: rule_span,
                }])?;

                let mut provided_fields = HashMap::new();
                for (name, expr) in fields {
                    let field_schema = schema_entity.fields.get(name).ok_or_else(|| vec![AnalyzerError {
                        code: "L010",
                        message: format!("Unknown field '{}' on entity '{}'", name, entity),
                        span: rule_span,
                    }])?;

                    let ty = self.infer_type(expr, None, None).map_err(|e| vec![e])?;
                    if ty != field_schema.ty {
                        return Err(vec![AnalyzerError {
                            code: "L002",
                            message: format!("Type mismatch for field '{}'", name),
                            span: rule_span,
                        }]);
                    }
                    provided_fields.insert(name.clone(), ());
                }

                for (name, field) in &schema_entity.fields {
                    if !field.is_derived && !provided_fields.contains_key(name) {
                        return Err(vec![AnalyzerError {
                            code: "L007",
                            message: format!("Stored field '{}' missing on entity creation", name),
                            span: rule_span,
                        }]);
                    }
                }
                Ok(())
            }
            Action::Delete(instance) => {
                // Simplified: just check if an entity with this name exists in schema
                if !self.schema.entities.contains_key(instance) {
                    return Err(vec![AnalyzerError {
                        code: "L001",
                        message: format!("Unknown instance: {}", instance),
                        span: rule_span,
                    }]);
                }
                Ok(())
            }
            Action::Alert(_) => Ok(()),
            Action::Write { target, value } => {
                let entity_name = match self.instances.get(&target.instance) {
                    Some(LuminaType::Entity(e)) => e,
                    _ => &target.instance,
                };
                // L038: write actions can only target external entities
                if let Some(entity_schema) = self.schema.get_entity(entity_name) {
                    if !entity_schema.is_external {
                        return Err(vec![AnalyzerError {
                            code: "L038",
                            message: format!("write action can only target external entities, '{}' is not external", entity_name),
                            span: target.span,
                        }]);
                    }
                    if let Some(field_schema) = entity_schema.fields.get(&target.field) {
                        let val_ty = self.infer_type(value, None, None).map_err(|e| vec![e])?;
                        if val_ty != field_schema.ty {
                            return Err(vec![AnalyzerError {
                                code: "L002",
                                message: "Type mismatch in write".to_string(),
                                span: target.span,
                            }]);
                        }
                    } else {
                        return Err(vec![AnalyzerError {
                            code: "L010",
                            message: format!("Unknown field '{}' on entity '{}'", target.field, entity_name),
                            span: target.span,
                        }]);
                    }
                } else {
                    return Err(vec![AnalyzerError {
                        code: "L001",
                        message: format!("Unknown entity: {}", entity_name),
                        span: target.span,
                    }]);
                }
                Ok(())
            }
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use lumina_parser::parse;
    use super::*;

    fn analyze_source(source: &str) -> Result<AnalyzedProgram, Vec<AnalyzerError>> {
        let program = parse(source).map_err(|e| vec![AnalyzerError {
            code: "LEX/PARSE",
            message: e.to_string(),
            span: Span::default(),
        }])?;
        Analyzer::new().analyze(program)
    }

    #[test]
    fn test_prev_analyzer_errors() {
        // L024: cannot use prev on derived field
        let src1 = "entity E { val: Number d := val * 2  bad := prev(d) }";
        let errs1 = analyze_source(src1).unwrap_err();
        assert!(errs1.iter().any(|e| e.code == "L024"));

        // L025: no nested prev (this fails at the syntax level because prev is a keyword, not an identifier)
        let src2 = "entity E { val: Number bad := prev(prev(val)) }";
        let errs2 = analyze_source(src2).unwrap_err();
        assert!(errs2.iter().any(|e| e.code == "LEX/PARSE"));
    }

    #[test]
    fn test_valid_entity_with_derived_fields() {
        let source = "entity Person { age: Number isAdult := age >= 18 }";
        let res = analyze_source(source).expect("analysis should succeed");
        assert!(res.schema.get_entity("Person").is_some());
        let age_id = res.graph.get_node("Person", "age").unwrap();
        let adult_id = res.graph.get_node("Person", "isAdult").unwrap();
        assert!(res.graph.dependents[age_id as usize].contains(&adult_id));
    }

    #[test]
    fn test_circular_dependency() {
        let source = "entity A { a := b b := a }";
        let errs = analyze_source(source).err().unwrap();
        assert!(errs.iter().any(|e| e.code == "L004"));
    }

    #[test]
    fn test_type_mismatch_in_derived_field() {
        let source = "entity A { name: Text age := name + 1 }";
        let errs = analyze_source(source).err().unwrap();
        assert!(errs.iter().any(|e| e.code == "L002"));
    }

    #[test]
    fn test_update_derived_field() {
        let source = "entity A { x := 1 } rule \"test\" { when true then update A.x to 2 }";
        let errs = analyze_source(source).err().unwrap();
        assert!(errs.iter().any(|e| e.code == "L003"));
    }

    #[test]
    fn test_unknown_field_access() {
        let source = "entity A { x: Number } rule \"test\" { when true then update A.y to 2 }";
        let errs = analyze_source(source).err().unwrap();
        assert!(errs.iter().any(|e| e.code == "L010"));
    }

    #[test]
    fn test_valid_rule_with_becomes_condition() {
        let source = "entity A { x: Boolean } rule \"test\" { when A.x becomes true then show \"changed\" }";
        let res = analyze_source(source).expect("analysis should succeed");
        assert_eq!(res.program.statements.len(), 2);
    }

    // ── Phase 2: L035–L042 tests ──────────────────────────────────────────

    #[test]
    fn test_l036_ref_to_nonexistent_entity() {
        let source = "entity A { link: ref NonExistent }";
        let errs = analyze_source(source).unwrap_err();
        assert!(errs.iter().any(|e| e.code == "L036"), "expected L036, got: {:?}", errs);
    }

    #[test]
    fn test_valid_ref_declaration() {
        let source = "entity Cooling { ok: Boolean } entity Server { cooling: ref Cooling }";
        let res = analyze_source(source).expect("valid ref should pass");
        let server = res.schema.get_entity("Server").unwrap();
        assert!(server.fields.contains_key("cooling"));
    }

    #[test]
    fn test_l038_write_on_non_external_entity() {
        let source = "entity Local { x: Number } rule \"w\" { when true then write Local.x = 5 }";
        let errs = analyze_source(source).unwrap_err();
        assert!(errs.iter().any(|e| e.code == "L038"), "expected L038, got: {:?}", errs);
    }

    #[test]
    fn test_l042_invalid_timestamp_accessor() {
        let source = "entity S { ts: Timestamp bad := ts.foo }";
        let errs = analyze_source(source).unwrap_err();
        assert!(errs.iter().any(|e| e.code == "L042"), "expected L042, got: {:?}", errs);
    }

    #[test]
    fn test_now_returns_timestamp() {
        // now() in a rule condition is valid and should infer as Timestamp
        let source = "entity S { ts: Timestamp } rule \"r\" { when true then update S.ts to now() }";
        let res = analyze_source(source);
        // This should either succeed or fail with a type mismatch (Timestamp == Timestamp => ok)
        assert!(res.is_ok(), "now() should return Timestamp type, got: {:?}", res.err());
    }

    #[test]
    fn test_age_returns_number() {
        // ts.age should return Number (seconds since timestamp)
        let source = "entity S { ts: Timestamp stale := ts.age > 60 }";
        let res = analyze_source(source);
        assert!(res.is_ok(), ".age should return Number, got: {:?}", res.err());
    }
}
