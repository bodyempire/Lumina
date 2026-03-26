use std::collections::{HashMap, HashSet};
use std::time::Instant;
use crate::aggregate::AggregateStore;
use lumina_analyzer::types::Schema;
use lumina_analyzer::graph::DependencyGraph;
use lumina_parser::ast::*;
use crate::value::Value;
use crate::store::{EntityStore, Instance};
use crate::snapshot::{SnapshotStack, PropResult, FiredEvent, RollbackResult, Diagnostic};
use crate::RuntimeError;
use crate::rules;
use crate::timers::TimerHeap;
use crate::adapter::LuminaAdapter;
use crate::fleet::FleetState;

pub const MAX_DEPTH: usize = 100;

pub struct Evaluator {
    pub schema:    Schema,
    pub graph:     DependencyGraph,
    pub rules:     Vec<RuleDecl>,
    pub store:     EntityStore,
    pub snapshots: SnapshotStack,
    pub env:       HashMap<String, Value>,
    pub instances: HashMap<String, String>,
    pub derived_exprs: HashMap<(String, String), Expr>,
    pub functions: HashMap<String, FnDecl>,
    pub timers:    TimerHeap,
    pub adapters:  Vec<Box<dyn LuminaAdapter>>,
    pub prev_store: Option<EntityStore>,
    pub fleet_state: FleetState,
    prev_fleet_any: HashMap<(String, String), bool>,
    prev_fleet_all: HashMap<(String, String), bool>,
    depth:         usize,
    fired_this_cycle: HashSet<String>,
    output:        Vec<String>,
    pub cooldown_map: HashMap<(String, String), f64>,
    pub rule_active: HashMap<(String, String), bool>,
    pub agg_store: AggregateStore,
    pub now:       f64,
}
impl Evaluator {
    pub fn get_output(&self) -> &[String] {
        &self.output
    }

    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    pub fn new(schema: Schema, graph: DependencyGraph, rules: Vec<RuleDecl>) -> Self {
        let mut timers = TimerHeap::new();
        timers.register_every_rules(&rules);
        Self {
            schema, graph, rules,
            store: EntityStore::new(),
            snapshots: SnapshotStack::new(),
            env: HashMap::new(),
            instances: HashMap::new(),
            derived_exprs: HashMap::new(),
            functions: HashMap::new(),
            timers,
            adapters: Vec::new(),
            prev_store: None,
            fleet_state: FleetState::new(),
            prev_fleet_any: HashMap::new(),
            prev_fleet_all: HashMap::new(),
            depth: 0,
            fired_this_cycle: HashSet::new(),
            output: Vec::new(),
            cooldown_map: HashMap::new(),
            rule_active: HashMap::new(),
            agg_store: AggregateStore::new(),
            now: 0.0,
        }
    }

    /// Creates an empty evaluator with no entities, rules, or instances.
    /// Used by the REPL - statements are added one at a time via exec_statement().
    pub fn new_empty() -> Self {
        Self {
            schema: Schema::new(),
            graph: DependencyGraph::new(),
            rules: Vec::new(),
            store: EntityStore::new(),
            snapshots: SnapshotStack::new(),
            env: HashMap::new(),
            instances: HashMap::new(),
            derived_exprs: HashMap::new(),
            functions: HashMap::new(),
            timers: TimerHeap::new(),
            adapters: Vec::new(),
            prev_store: None,
            fleet_state: FleetState::new(),
            prev_fleet_any: HashMap::new(),
            prev_fleet_all: HashMap::new(),
            depth: 0,
            fired_this_cycle: HashSet::new(),
            output: Vec::new(),
            cooldown_map: HashMap::new(),
            rule_active: HashMap::new(),
            agg_store: AggregateStore::new(),
            now: 0.0,
        }
    }

    /// Describe all declared entities as a human-readable string.
    /// Used by :schema REPL command.
    pub fn describe_schema(&self) -> String {
        if self.schema.entities.is_empty() {
            return "(no entities declared)".into();
        }
        self.schema.entities.iter()
            .map(|(name, ent)| {
                let fields = ent.fields.iter()
                    .map(|(n, f)| format!("{}: {:?}", n, f.ty))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("entity {} {{ {} }}", name, fields)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn should_fire(&self, rule_name: &str, instance_name: &str, cooldown: &Duration) -> bool {
        if let Some(last_fired) = self.cooldown_map.get(&(rule_name.to_string(), instance_name.to_string())) {
            (self.now - *last_fired) >= (cooldown.to_seconds() * 1000.0)
        } else {
            true
        }
    }

    pub fn record_firing(&mut self, rule_name: &str, instance_name: &str) {
        self.cooldown_map.insert((rule_name.to_string(), instance_name.to_string()), self.now);
    }

    pub fn register_derived(&mut self, entity: &str, field: &str, expr: Expr) {
        self.derived_exprs.insert((entity.to_string(), field.to_string()), expr);
    }

    /// Register an external entity adapter.
    pub fn register_adapter(&mut self, a: Box<dyn LuminaAdapter>) {
        self.adapters.push(a);
    }

    pub fn drain_output(&mut self) -> Vec<String> {
        std::mem::take(&mut self.output)
    }

    // ── Expression evaluator ──────────────────────────────

    pub fn eval_expr(&self, expr: &Expr, ctx: Option<&str>) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::Text(s) => Ok(Value::Text(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),

            Expr::Ident(name) => {
                if let Some(inst) = ctx {
                    if let Some(instance) = self.store.get(inst) {
                        if let Some(val) = instance.get(name) {
                            return Ok(val.clone());
                        }
                    }
                }
                if let Some(val) = self.env.get(name) {
                    return Ok(val.clone());
                }
                if let Some(val) = self.agg_store.get(name, "") {
                    return Ok(val.clone());
                }
                Err(RuntimeError::R001 { instance: name.clone() })
            }

            Expr::FieldAccess { obj, field, .. } => {
                let mut inst_name = match obj.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    // Support chained ref traversal: a.b.c
                    Expr::FieldAccess { .. } => {
                        let ref_val = self.eval_expr(obj, ctx)?;
                        match ref_val {
                            // .age on a Timestamp value
                            Value::Timestamp(ts) => {
                                if field == "age" {
                                    return Ok(Value::Number(self.now - ts));
                                }
                                return Err(RuntimeError::R005 { instance: "Timestamp".into(), field: field.clone() });
                            }
                            _ => return Err(RuntimeError::R001 { instance: format!("{:?}", obj) }),
                        }
                    }
                    _ => return Err(RuntimeError::R001 { instance: format!("{:?}", obj) }),
                };

                // Bug Fix: If inst_name is an entity name, and it matches the current context's entity,
                // then resolve it to the context instance.
                if let Some(ctx_inst) = ctx {
                    if let Some(ctx_ent) = self.instances.get(ctx_inst) {
                        if &inst_name == ctx_ent {
                            inst_name = ctx_inst.to_string();
                        }
                    }
                }

                if let Some(val) = self.agg_store.get(&inst_name, field) {
                    return Ok(val.clone());
                }

                let instance = self.store.get(&inst_name)
                    .ok_or(RuntimeError::R001 { instance: inst_name.clone() })?;

                let val = instance.get(field).cloned()
                    .ok_or(RuntimeError::R005 { instance: inst_name.clone(), field: field.clone() })?;

                // .age accessor on a Timestamp field value
                if field == "age" {
                    // This branch won't fire — age is not a stored field.
                    // The Timestamp .age case is handled via the match on Value::Timestamp below.
                    return Ok(val);
                }

                // If the resolved value is a Timestamp and we're accessing .age on it,
                // that's handled by chained FieldAccess above. For single-level access,
                // check if the caller wants .age on the result:
                match &val {
                    Value::Timestamp(ts) if field == "age" => {
                        Ok(Value::Number(self.now - ts))
                    }
                    _ => Ok(val),
                }
            }

            Expr::Binary { op, left, right, .. } => {
                // Short-circuit
                if *op == BinOp::And {
                    let l = self.eval_expr(left, ctx)?;
                    if l == Value::Bool(false) { return Ok(Value::Bool(false)); }
                    let r = self.eval_expr(right, ctx)?;
                    return Ok(Value::Bool(l == Value::Bool(true) && r == Value::Bool(true)));
                }
                if *op == BinOp::Or {
                    let l = self.eval_expr(left, ctx)?;
                    if l == Value::Bool(true) { return Ok(Value::Bool(true)); }
                    let r = self.eval_expr(right, ctx)?;
                    return Ok(Value::Bool(r == Value::Bool(true)));
                }

                let l = self.eval_expr(left, ctx)?;
                let r = self.eval_expr(right, ctx)?;
                self.apply_binop(op, l, r)
            }

            Expr::Unary { op, operand, .. } => {
                let v = self.eval_expr(operand, ctx)?;
                match op {
                    UnOp::Neg => match v { Value::Number(n) => Ok(Value::Number(-n)), _ => Ok(Value::Number(0.0)) },
                    UnOp::Not => match v { Value::Bool(b) => Ok(Value::Bool(!b)), _ => Ok(Value::Bool(false)) },
                }
            }

            Expr::If { cond, then_, else_, .. } => {
                if self.eval_expr(cond, ctx)? == Value::Bool(true) {
                    self.eval_expr(then_, ctx)
                } else {
                    self.eval_expr(else_, ctx)
                }
            }

            Expr::InterpolatedString(segments) => {
                let mut out = String::new();
                for seg in segments {
                    match seg {
                        StringSegment::Literal(s) => out.push_str(s),
                        StringSegment::Expr(e) => {
                            let v = self.eval_expr(e, ctx)?;
                            out.push_str(&v.to_string());
                        }
                    }
                }
                Ok(Value::Text(out))
            }

            Expr::Call { name, args, .. } => {
                // Built-in list functions (checked before user fn_defs)
                match name.as_str() {
                    "len" => {
                        let list = self.eval_to_list(&args[0], ctx)?;
                        return Ok(Value::Number(list.len() as f64));
                    }
                    "min" => {
                        let list = self.eval_to_num_list(&args[0], ctx)?;
                        if list.is_empty() { return Err(RuntimeError::R004 { index: 0, len: 0 }); }
                        return Ok(Value::Number(list.iter().cloned().fold(f64::INFINITY, f64::min)));
                    }
                    "max" => {
                        let list = self.eval_to_num_list(&args[0], ctx)?;
                        if list.is_empty() { return Err(RuntimeError::R004 { index: 0, len: 0 }); }
                        return Ok(Value::Number(list.iter().cloned().fold(f64::NEG_INFINITY, f64::max)));
                    }
                    "sum" => {
                        let list = self.eval_to_num_list(&args[0], ctx)?;
                        return Ok(Value::Number(list.iter().sum()));
                    }
                    "append" => {
                        let mut list = self.eval_to_list(&args[0], ctx)?;
                        let val = self.eval_expr(&args[1], ctx)?;
                        list.push(val);
                        return Ok(Value::List(list));
                    }
                    "head" => {
                        let list = self.eval_to_list(&args[0], ctx)?;
                        if list.is_empty() { return Err(RuntimeError::R004 { index: 0, len: 0 }); }
                        return Ok(list[0].clone());
                    }
                    "tail" => {
                        let list = self.eval_to_list(&args[0], ctx)?;
                        if list.is_empty() { return Err(RuntimeError::R004 { index: 0, len: 0 }); }
                        return Ok(Value::List(list[1..].to_vec()));
                    }
                    "at" => {
                        let list = self.eval_to_list(&args[0], ctx)?;
                        let idx = self.eval_expr(&args[1], ctx)?.as_number()
                            .ok_or(RuntimeError::R002)? as usize;
                        if idx >= list.len() {
                            return Err(RuntimeError::R004 { index: idx, len: list.len() });
                        }
                        return Ok(list[idx].clone());
                    }
                    "now" => {
                        return Ok(Value::Timestamp(self.now));
                    }
                    _ => {} // Fall through to user-defined fn lookup
                }
                let decl = self.functions.get(name)
                    .ok_or(RuntimeError::R002)?
                    .clone();
                let arg_vals: Vec<Value> = args.iter()
                    .map(|a| self.eval_expr(a, ctx))
                    .collect::<Result<_, _>>()?;
                let mut local: HashMap<String, Value> = HashMap::new();
                for (param, val) in decl.params.iter().zip(arg_vals) {
                    local.insert(param.name.clone(), val);
                }
                self.eval_expr_local(&decl.body, &local)
            }
            Expr::ListLiteral(elems) => {
                let vals: Vec<Value> = elems.iter()
                    .map(|e| self.eval_expr(e, ctx))
                    .collect::<Result<_, _>>()?;
                Ok(Value::List(vals))
            }
            Expr::Index { list, index, .. } => {
                let list_val = self.eval_to_list(list, ctx)?;
                let idx = self.eval_expr(index, ctx)?.as_number()
                    .ok_or(RuntimeError::R002)? as usize;
                if idx >= list_val.len() {
                    return Err(RuntimeError::R004 { index: idx, len: list_val.len() });
                }
                Ok(list_val[idx].clone())
            }
            Expr::Prev { field, .. } => {
                let inst_name = ctx.ok_or(RuntimeError::R001 { instance: "global".into() })?;
                
                // First check prev_store
                if let Some(prev) = &self.prev_store {
                    if let Some(instance) = prev.get(inst_name) {
                        return instance.get(field).cloned()
                            .ok_or(RuntimeError::R005 { instance: inst_name.to_string(), field: field.clone() });
                    }
                }
                
                // Fallback to current store if prev_store is not set (e.g. initialization)
                let instance = self.store.get(inst_name)
                    .ok_or(RuntimeError::R001 { instance: inst_name.to_string() })?;
                instance.get(field).cloned()
                    .ok_or(RuntimeError::R005 { instance: inst_name.to_string(), field: field.clone() })
            }
        }
    }

    // ── Function evaluation ───────────────────────────────

    fn apply_binop(&self, op: &BinOp, l: Value, r: Value) -> Result<Value, RuntimeError> {
        match op {
            BinOp::Add => match (l, r) { (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)), _ => Ok(Value::Number(0.0)) },
            BinOp::Sub => match (l, r) { (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)), _ => Ok(Value::Number(0.0)) },
            BinOp::Mul => match (l, r) { (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)), _ => Ok(Value::Number(0.0)) },
            BinOp::Div => match (l, r) {
                (Value::Number(a), Value::Number(b)) => {
                    if b == 0.0 { Err(RuntimeError::R002) } else { Ok(Value::Number(a / b)) }
                }
                _ => Ok(Value::Number(0.0)),
            },
            BinOp::Mod => match (l, r) {
                (Value::Number(a), Value::Number(b)) => {
                    if b == 0.0 { Err(RuntimeError::R002) } else { Ok(Value::Number(a % b)) }
                }
                _ => Ok(Value::Number(0.0)),
            },
            BinOp::Eq => Ok(Value::Bool(l == r)),
            BinOp::Ne => Ok(Value::Bool(l != r)),
            BinOp::Gt  => match (l, r) { (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),  _ => Ok(Value::Bool(false)) },
            BinOp::Lt  => match (l, r) { (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),  _ => Ok(Value::Bool(false)) },
            BinOp::Ge  => match (l, r) { (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)), _ => Ok(Value::Bool(false)) },
            BinOp::Le  => match (l, r) { (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)), _ => Ok(Value::Bool(false)) },
            BinOp::And | BinOp::Or => unreachable!(),
        }
    }

    fn eval_expr_local(&self, expr: &Expr, locals: &HashMap<String, Value>) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Ident(name) => locals.get(name)
                .cloned()
                .ok_or(RuntimeError::R005 { instance: name.clone(), field: name.clone() }),
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::Text(s) => Ok(Value::Text(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Binary { op, left, right, .. } => {
                if *op == BinOp::And {
                    let l = self.eval_expr_local(left, locals)?;
                    if l == Value::Bool(false) { return Ok(Value::Bool(false)); }
                    let r = self.eval_expr_local(right, locals)?;
                    return Ok(Value::Bool(r == Value::Bool(true)));
                }
                if *op == BinOp::Or {
                    let l = self.eval_expr_local(left, locals)?;
                    if l == Value::Bool(true) { return Ok(Value::Bool(true)); }
                    let r = self.eval_expr_local(right, locals)?;
                    return Ok(Value::Bool(r == Value::Bool(true)));
                }
                let l = self.eval_expr_local(left, locals)?;
                let r = self.eval_expr_local(right, locals)?;
                self.apply_binop(op, l, r)
            }
            Expr::If { cond, then_, else_, .. } => {
                let c = self.eval_expr_local(cond, locals)?;
                if c == Value::Bool(true) {
                    self.eval_expr_local(then_, locals)
                } else {
                    self.eval_expr_local(else_, locals)
                }
            }
            Expr::InterpolatedString(segments) => {
                let mut out = String::new();
                for seg in segments {
                    match seg {
                        StringSegment::Literal(s) => out.push_str(s),
                        StringSegment::Expr(e) => {
                            let v = self.eval_expr_local(e, locals)?;
                            out.push_str(&v.to_string());
                        }
                    }
                }
                Ok(Value::Text(out))
            }
            Expr::ListLiteral(elems) => {
                let vals: Vec<Value> = elems.iter()
                    .map(|e| self.eval_expr_local(e, locals))
                    .collect::<Result<_, _>>()?;
                Ok(Value::List(vals))
            }
            Expr::Index { list, index, .. } => {
                let list_val = self.eval_expr_local(list, locals)?;
                let items = match list_val {
                    Value::List(l) => l,
                    _ => return Err(RuntimeError::R002),
                };
                let idx = self.eval_expr_local(index, locals)?.as_number()
                    .ok_or(RuntimeError::R002)? as usize;
                if idx >= items.len() {
                    return Err(RuntimeError::R004 { index: idx, len: items.len() });
                }
                Ok(items[idx].clone())
            }
            _ => Err(RuntimeError::R002), // unsupported expr in fn body
        }
    }

    // ── Statement executor ────────────────────────────────

    pub fn exec_statement(&mut self, stmt: &Statement) -> Result<Vec<FiredEvent>, RuntimeError> {
        match stmt {
            Statement::Entity(_) | Statement::ExternalEntity(_) | Statement::Rule(_) => Ok(vec![]),
            Statement::Aggregate(decl) => {
                self.agg_store.register(decl.clone());
                self.agg_store.recompute(&self.store);
                Ok(vec![])
            }
            Statement::Fn(decl) => {
                self.functions.insert(decl.name.clone(), decl.clone());
                Ok(vec![])
            }
            Statement::Let(ls) => {
                match &ls.value {
                    LetValue::Expr(expr) => {
                        let val = self.eval_expr(expr, None)?;
                        self.env.insert(ls.name.clone(), val);
                        Ok(vec![])
                    }
                    LetValue::EntityInit(init) => {
                        let mut fields = HashMap::new();
                        for (name, expr) in &init.fields {
                            fields.insert(name.clone(), self.eval_expr(expr, None)?);
                        }
                        let inst_name = ls.name.clone();
                        let entity_name = init.entity_name.clone();
                        self.instances.insert(inst_name.clone(), entity_name.clone());
                        self.store.insert(inst_name.clone(), Instance::new(&entity_name, fields));
                        // Compute derived fields for the new instance
                        self.propagate_derived(&inst_name, &entity_name)?;
                        self.store.commit_all();
                        
                        // Initial rule evaluation for this new instance
                        self.evaluate_rules(&inst_name)
                    }
                }
            }
            Statement::Action(a) => self.exec_action(a, None),
            Statement::Import(_) => Ok(vec![])
        }
    }

    // ── Action executor ───────────────────────────────────

    pub fn exec_action(&mut self, action: &Action, ctx: Option<&str>) -> Result<Vec<FiredEvent>, RuntimeError> {
        match action {
            Action::Show(expr) => {
                let val = self.eval_expr(expr, ctx)?;
                let s = val.to_string();
                println!("{}", s);
                self.output.push(s);
                Ok(vec![])
            }
            Action::Update { target, value } => {
                let val = self.eval_expr(value, ctx)?;
                let mut inst_name = target.instance.clone();
                if let Some(ctx_inst) = ctx {
                    if let Some(ctx_ent) = self.instances.get(ctx_inst) {
                        if ctx_ent == &inst_name {
                            inst_name = ctx_inst.to_string();
                        }
                    }
                }
                self.apply_update(&inst_name, &target.field, val)
            }
            Action::Create { entity, fields } => {
                let mut fv = HashMap::new();
                for (name, expr) in fields {
                    fv.insert(name.clone(), self.eval_expr(expr, ctx)?);
                }
                let count = self.store.all_of_entity(entity).count();
                let inst_name = format!("{}_{}", entity.to_lowercase(), count + 1);
                self.instances.insert(inst_name.clone(), entity.clone());
                self.store.insert(inst_name, Instance::new(entity, fv));
                Ok(vec![])
            }
            Action::Delete(name) => {
                let mut inst_name = name.clone();
                if let Some(ctx_inst) = ctx {
                    if let Some(ctx_ent) = self.instances.get(ctx_inst) {
                        if ctx_ent == &inst_name {
                            inst_name = ctx_inst.to_string();
                        }
                    }
                }
                self.store.remove(&inst_name).ok_or(RuntimeError::R001 { instance: inst_name.clone() })?;
                Ok(vec![])
            }
            Action::Alert(alert_action) => {
                let severity = self.eval_expr(&alert_action.severity, ctx)?
                    .to_string();
                let message = self.eval_expr(&alert_action.message, ctx)?
                    .to_string();
                let source = alert_action.source.as_ref()
                    .and_then(|e| self.eval_expr(e, ctx).ok())
                    .map(|v| v.to_string())
                    .unwrap_or_default();

                // Validate severity
                match severity.as_str() {
                    "info" | "warning" | "critical" | "resolved" => {}
                    _ => return Err(RuntimeError::R002),
                }

                // Output for development visibility
                let line = format!("[ALERT:{}] {} -- {}", severity, source, message);
                println!("{}", line);
                self.output.push(line);

                Ok(vec![FiredEvent {
                    rule: ctx.unwrap_or("").to_string(),
                    instance: source,
                    severity,
                    message,
                    ts: self.now,
                }])
            }
            Action::Write { target, value } => {
                let val = self.eval_expr(value, ctx)?;
                let mut inst_name = target.instance.clone();
                if let Some(ctx_inst) = ctx {
                    if let Some(ctx_ent) = self.instances.get(ctx_inst) {
                        if ctx_ent == &inst_name {
                            inst_name = ctx_inst.to_string();
                        }
                    }
                }
                // Dispatch to adapter if one is registered for this entity
                let entity_name = self.instances.get(&inst_name)
                    .cloned()
                    .unwrap_or_else(|| inst_name.clone());
                let mut dispatched = false;
                for adapter in &mut self.adapters {
                    if adapter.entity_name() == entity_name {
                        adapter.on_write(&target.field, &val);
                        dispatched = true;
                        break;
                    }
                }
                // Also update the local store so the state is consistent
                if self.store.get(&inst_name).is_some() {
                    self.apply_update(&inst_name, &target.field, val)
                } else if dispatched {
                    Ok(vec![])
                } else {
                    self.apply_update(&inst_name, &target.field, val)
                }
            }
        }
    }

    // ── Core update + propagation ─────────────────────────

    pub fn apply_update(
        &mut self,
        instance_name: &str,
        field_name: &str,
        new_value: Value,
    ) -> Result<Vec<FiredEvent>, RuntimeError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(RuntimeError::R003 { depth: self.depth });
        }

        // Capture pre-update state for `prev()` expressions
        if self.depth == 1 {
            self.prev_store = Some(self.store.clone());
        }

        let snap = self.snapshots.take(&self.store);
        self.snapshots.push(snap);
        self.agg_store.recompute(&self.store);
        
        let entity_name = self.store.get(instance_name)
            .ok_or(RuntimeError::R001 { instance: instance_name.to_string() })?
            .entity_name.clone();
        // Check if field is derived (cannot be manually updated)
        if self.derived_exprs.contains_key(&(entity_name.clone(), field_name.to_string())) {
            self.snapshots.pop();
            self.depth -= 1;
            return Err(RuntimeError::R009 { field: field_name.to_string() });
        }

        // Check @range
        if let Value::Number(n) = &new_value {
            if let Some(fs) = self.schema.get_field(&entity_name, field_name) {
                if let Some((min, max)) = fs.metadata.range {
                    if *n < min || *n > max {
                        self.snapshots.pop();
                        self.depth -= 1;
                        return Err(RuntimeError::R006 { field: field_name.into(), value: *n, min, max });
                    }
                }
            }
        }

        // Capture old Boolean value for fleet tracking
        let old_bool = self.store.get(instance_name)
            .and_then(|inst| inst.get(field_name))
            .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None });

        // Apply
        self.store.get_mut(instance_name)
            .ok_or(RuntimeError::R001 { instance: instance_name.to_string() })?
            .set(field_name, new_value.clone());

        // Update fleet state for Boolean fields
        if let Value::Bool(new_b) = &new_value {
            let total = self.store.all_of_entity(&entity_name).count();
            self.fleet_state.update(
                &entity_name, field_name,
                old_bool.unwrap_or(false), *new_b, total,
            );
        }

        // Write-back to external entity adapters
        for a in &mut self.adapters {
            if a.entity_name() == entity_name {
                a.on_write(field_name, &new_value);
            }
        }

        // Propagate derived fields
        if let Err(e) = self.propagate_derived(instance_name, &entity_name) {
            let snap = self.snapshots.pop().unwrap();
            self.store = snap.store;
            self.depth -= 1;
            return Err(e);
        }

        // Evaluate rules
        let all_events = self.evaluate_rules(instance_name)?;

        // Only commit at outermost level to prevent re-triggering becomes
        if self.depth == 1 {
            self.store.commit_all();
            self.fired_this_cycle.clear();
        }
        self.snapshots.pop();
        self.depth -= 1;
        Ok(all_events)
    }

    fn evaluate_rules(&mut self, instance_name: &str) -> Result<Vec<FiredEvent>, RuntimeError> {
        let mut all_events = Vec::new();
        let rules_clone = self.rules.clone();
        for rule in &rules_clone {
            // FIX Issue 6: Check if instance was deleted by a previous rule in this cycle
            if self.store.get(instance_name).is_none() {
                break; // Instance is gone, stop evaluating rules for it
            }

            match &rule.trigger {
                RuleTrigger::When(conditions) => {
                    let active_key = (rule.name.clone(), instance_name.to_string());
                    // All conditions in the compound trigger must be met
                    let all_met = conditions.iter().all(|c| {
                        rules::condition_is_met(self, c, instance_name, true).unwrap_or(false)
                    });
                    match all_met {
                        true => {
                            let fire_key = format!("{}::{}", rule.name, instance_name);
                            if self.fired_this_cycle.contains(&fire_key) {
                                // Mark as active even if we skip firing
                                self.rule_active.insert(active_key, true);
                                continue;
                            }
                            // Use the for_duration from the first condition if present
                            let for_duration = conditions.first().and_then(|c| c.for_duration.as_ref());
                            if let Some(dur) = for_duration {
                                let _ = self.timers.start_for_timer(
                                    &rule.name, instance_name, dur.to_seconds(),
                                );
                            } else {
                                if let Some(cd) = &rule.cooldown {
                                    if !self.should_fire(&rule.name, instance_name, cd) {
                                        self.rule_active.insert(active_key, true);
                                        continue;
                                    }
                                }
                                self.record_firing(&rule.name, instance_name);

                                self.fired_this_cycle.insert(fire_key);
                                 for action in &rule.actions {
                                    let evts = self.exec_action(action, Some(instance_name))?;
                                    all_events.extend(evts);
                                }
                                all_events.push(FiredEvent {
                                    rule: rule.name.clone(),
                                    instance: instance_name.to_string(),
                                    severity: "info".to_string(),
                                    message: format!("Rule '{}' fired", rule.name),
                                    ts: self.now,
                                });
                            }
                            self.rule_active.insert(active_key, true);
                        }
                        false => {
                            self.timers.cancel_for_timer(&rule.name, instance_name);
                            // on_clear: if rule was previously active, fire on_clear actions
                            let was_active = self.rule_active.get(&active_key).copied().unwrap_or(false);
                            if was_active {
                                self.rule_active.insert(active_key, false);
                                if let Some(clear_actions) = &rule.on_clear {
                                    let clear_actions = clear_actions.clone();
                                    for action in &clear_actions {
                                        let evts = self.exec_action(action, Some(instance_name))?;
                                        all_events.extend(evts);
                                    }
                                    all_events.push(FiredEvent {
                                        rule: format!("{}_clear", rule.name),
                                        instance: instance_name.to_string(),
                                        severity: "resolved".to_string(),
                                        message: format!("Rule '{}' cleared", rule.name),
                                        ts: self.now,
                                    });
                                }
                            }
                        }
                    }
                }
                RuleTrigger::Any(fc) => {
                    let key = (fc.entity.clone(), fc.field.clone());
                    let target = matches!(&fc.becomes, Expr::Bool(true));
                    let now_met = if target {
                        self.fleet_state.any_true(&fc.entity, &fc.field)
                    } else {
                        !self.fleet_state.all_true(&fc.entity, &fc.field)
                    };
                    let prev = self.prev_fleet_any.get(&key).copied().unwrap_or(false);
                    // Edge detection: fire only on rising edge
                    if now_met && !prev {
                        let fire_key = format!("{}::fleet_any", rule.name);
                        if !self.fired_this_cycle.contains(&fire_key) {
                            self.fired_this_cycle.insert(fire_key);
                            for action in &rule.actions {
                                let evts = self.exec_action(action, None)?;
                                all_events.extend(evts);
                            }
                            all_events.push(FiredEvent {
                                rule: rule.name.clone(),
                                instance: "fleet".to_string(),
                                severity: "info".to_string(),
                                message: format!("Fleet any trigger fired for '{}'", rule.name),
                                ts: self.now,
                            });
                        }
                    }
                    self.prev_fleet_any.insert(key, now_met);
                }
                RuleTrigger::All(fc) => {
                    let key = (fc.entity.clone(), fc.field.clone());
                    let target = matches!(&fc.becomes, Expr::Bool(true));
                    let now_met = if target {
                        self.fleet_state.all_true(&fc.entity, &fc.field)
                    } else {
                        !self.fleet_state.any_true(&fc.entity, &fc.field)
                    };
                    let prev = self.prev_fleet_all.get(&key).copied().unwrap_or(false);
                    // Edge detection: fire only on rising edge
                    if now_met && !prev {
                        let fire_key = format!("{}::fleet_all", rule.name);
                        if !self.fired_this_cycle.contains(&fire_key) {
                            self.fired_this_cycle.insert(fire_key);
                            for action in &rule.actions {
                                let evts = self.exec_action(action, None)?;
                                all_events.extend(evts);
                            }
                            all_events.push(FiredEvent {
                                rule: rule.name.clone(),
                                instance: "fleet".to_string(),
                                severity: "info".to_string(),
                                message: format!("Fleet all trigger fired for '{}'", rule.name),
                                ts: self.now,
                            });
                        }
                    }
                    self.prev_fleet_all.insert(key, now_met);
                }
                RuleTrigger::Every(_) => {} // handled in tick()
            }
        }
        Ok(all_events)
    }

    /// Run a full sweep of all rules across all instances.
    /// Typically used after initialization to establish first stable state.
    pub fn recalculate_all_rules(&mut self) -> Result<Vec<FiredEvent>, RuntimeError> {
        let mut all_events = Vec::new();
        let instance_names: Vec<String> = self.store.all().map(|(n, _)| n.clone()).collect();
        for name in instance_names {
            let evts = self.evaluate_rules(&name)?;
            all_events.extend(evts);
        }
        self.store.commit_all();
        Ok(all_events)
    }

    fn propagate_derived(&mut self, instance_name: &str, entity_name: &str) -> Result<(), RuntimeError> {
        let mut derived: Vec<(String, String)> = self.derived_exprs.keys()
            .filter(|(ent, _)| ent == entity_name)
            .cloned()
            .collect();
        derived.sort_by_key(|(e, f)| self.graph.get_node(e, f).unwrap_or(u32::MAX));

        for (ent, field) in derived {
            if let Some(expr) = self.derived_exprs.get(&(ent.clone(), field.clone())).cloned() {
                // Capture old value for fleet tracking
                let old_val = self.store.get(instance_name).and_then(|inst| inst.get(&field)).cloned();
                
                let val = self.eval_expr(&expr, Some(instance_name))?;
                
                // Update fleet state if field is Boolean
                if let Value::Bool(new_b) = &val {
                    let old_b = if let Some(Value::Bool(b)) = old_val { b } else { false };
                    let total = self.store.all_of_entity(&ent).count();
                    self.fleet_state.update(&ent, &field, old_b, *new_b, total);
                }

                if let Some(inst) = self.store.get_mut(instance_name) {
                    inst.set(&field, val);
                }
            }
        }
        Ok(())
    }

    /// Execute all actions of a rule — helper for both immediate and timer-delayed firing
    fn exec_rule_actions(
        &mut self,
        rule: &RuleDecl,
        instance_name: &str,
    ) -> Result<Vec<FiredEvent>, RuntimeError> {
        let ctx = if instance_name.is_empty() { None } else { Some(instance_name) };
        let mut events = vec![];
        for action in &rule.actions {
            let evts = self.exec_action(action, ctx)?;
            events.extend(evts);
        }
        events.push(FiredEvent {
            rule: rule.name.clone(),
            instance: instance_name.to_string(),
            severity: "info".to_string(),
            message: format!("Timer callback for '{}'", rule.name),
            ts: self.now,
        });
        Ok(events)
    }

    /// Called periodically by the host — fires any elapsed for/every timers
    pub fn tick(&mut self) -> Result<Vec<FiredEvent>, RollbackResult> {
        let mut all_events = vec![];

        // ── Poll external entity adapters ──────────────────────────
        let updates: Vec<(String, String, Value)> = self.adapters
            .iter_mut()
            .flat_map(|a| {
                let name = a.entity_name().to_string();
                std::iter::from_fn(move || {
                    a.poll().map(|(f, v)| (name.clone(), f, v))
                }).collect::<Vec<_>>()
            }).collect();

        for (entity, field, value) in updates {
            if let Some(inst_name) = self.store.find_instance_of(&entity) {
                // sync_on filtering: only propagate if the field matches sync_on
                // (or if no sync_on is set). Non-sync fields are still stored.
                if let Some(entity_schema) = self.schema.get_entity(&entity) {
                    if let Some(ref sync_field) = entity_schema.sync_on {
                        if &field != sync_field {
                            // Store the value without triggering propagation
                            if let Some(inst) = self.store.get_mut(&inst_name) {
                                inst.set(&field, value);
                            }
                            continue;
                        }
                    }
                }
                let _ = self.apply_update(&inst_name, &field, value);
            }
        }

        // ── Fire elapsed `for` timers ──────────────────────────────────
        let elapsed = self.timers.drain_elapsed_for_timers();
        for timer in elapsed {
            let rule = self.rules.iter()
                .find(|r| r.name == timer.rule_name)
                .cloned();
            if let Some(rule) = rule {
                if let RuleTrigger::When(conditions) = &rule.trigger {
                    let still_true = conditions.iter().all(|c| {
                        rules::condition_is_met(self, c, &timer.instance_name, false).unwrap_or(false)
                    });
                    if still_true {
                        let snap = self.snapshots.take(&self.store);
                        match self.exec_rule_actions(&rule, &timer.instance_name) {
                            Ok(events) => {
                                self.store.commit_all();
                                all_events.extend(events);
                            }
                            Err(e) => {
                                self.store = snap.store;
                                return Err(RollbackResult {
                                    diagnostic: Diagnostic::from_runtime_error(
                                        e.code(), &e.message(),
                                        self.snapshots.current_version(),
                                        vec![rule.name.clone()],
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }

        // ── Fire due `every` timers ────────────────────────────────────
        let due_rules = self.timers.drain_due_every_timers();
        for rule_name in due_rules {
            let rule = self.rules.iter()
                .find(|r| r.name == rule_name)
                .cloned();
            if let Some(rule) = rule {
                let snap = self.snapshots.take(&self.store);
                match self.exec_rule_actions(&rule, "") {
                    Ok(events) => {
                        self.store.commit_all();
                        all_events.extend(events);
                    }
                    Err(e) => {
                        self.store = snap.store;
                        return Err(RollbackResult {
                            diagnostic: Diagnostic::from_runtime_error(
                                e.code(), &e.message(),
                                self.snapshots.current_version(),
                                vec![rule.name.clone()],
                            ),
                        });
                    }
                }
            }
        }

        Ok(all_events)
    }

    // ── Public API ────────────────────────────────────────

    pub fn apply_event(
        &mut self,
        instance_name: &str,
        field_name: &str,
        new_value: Value,
    ) -> Result<PropResult, RollbackResult> {
        match self.apply_update(instance_name, field_name, new_value) {
            Ok(events) => Ok(PropResult {
                success: true,
                events_fired: events,
                version: self.snapshots.current_version(),
            }),
            Err(e) => Err(RollbackResult {
                diagnostic: Diagnostic::from_runtime_error(
                    e.code(), &e.message(),
                    self.snapshots.current_version(), vec![],
                ),
            }),
        }
    }

    pub fn export_state(&self) -> serde_json::Value {
        let mut instances = serde_json::Map::new();
        for (name, instance) in self.store.all() {
            let mut fields = serde_json::Map::new();
            for (fname, val) in &instance.fields {
                fields.insert(fname.clone(), self.value_to_json(val));
            }
            instances.insert(name.clone(), serde_json::json!({
                "entity": instance.entity_name,
                "fields": fields,
                "active_alert": self.rule_active.iter().any(|((_, inst), active)| inst == name && *active),
            }));
        }
        serde_json::json!({
            "instances": instances,
            "stable": true,
            "version": self.snapshots.current_version()
        })
    }

    // ── List helpers ──────────────────────────────────────

    fn eval_to_list(&self, expr: &Expr, ctx: Option<&str>) -> Result<Vec<Value>, RuntimeError> {
        match self.eval_expr(expr, ctx)? {
            Value::List(l) => Ok(l),
            _ => Err(RuntimeError::R002),
        }
    }

    fn eval_to_num_list(&self, expr: &Expr, ctx: Option<&str>) -> Result<Vec<f64>, RuntimeError> {
        let list = self.eval_to_list(expr, ctx)?;
        list.into_iter()
            .map(|v| v.as_number().ok_or(RuntimeError::R002))
            .collect()
    }

    fn value_to_json(&self, val: &Value) -> serde_json::Value {
        match val {
            Value::Number(n) if n.fract() == 0.0 => serde_json::json!(*n as i64),
            Value::Number(n) => serde_json::json!(*n),
            Value::Text(s) => serde_json::json!(s),
            Value::Bool(b) => serde_json::json!(b),
            Value::List(items) => {
                let arr: Vec<serde_json::Value> = items.iter()
                    .map(|v| self.value_to_json(v))
                    .collect();
                serde_json::json!(arr)
            }
            Value::Timestamp(t) => serde_json::json!(*t),
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use lumina_lexer::token::Span;
    use lumina_analyzer::graph::DependencyGraph;
    use lumina_analyzer::types::Schema;

    fn empty_eval() -> Evaluator {
        Evaluator::new(Schema::new(), DependencyGraph::new(), vec![])
    }

    fn build_eval(source: &str) -> Evaluator {
        let program = lumina_parser::parse(source).expect("parse failed");
        let analyzed = lumina_analyzer::analyze(program, source, "<runtime-test>", true).expect("analysis failed");
        let mut rules = Vec::new();
        let mut derived = HashMap::new();
        for stmt in &analyzed.program.statements {
            match stmt {
                Statement::Rule(r) => rules.push(r.clone()),
                Statement::Entity(e) => {
                    for f in &e.fields {
                        if let Field::Derived(df) = f {
                            derived.insert((e.name.clone(), df.name.clone()), df.expr.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        let mut ev = Evaluator::new(analyzed.schema, analyzed.graph, rules);
        ev.derived_exprs = derived;
        ev.functions = analyzed.fn_defs;
        ev
    }

    #[test]
    fn test_function_evaluation() {
        let source = "
            fn double(x: Number) -> Number { x * 2 }
            entity Math { val: Number res := double(val) }
        ";
        let mut ev = build_eval(source);
        ev.store.insert("m1".to_string(), crate::store::Instance::new("Math", vec![("val".to_string(), Value::Number(10.0))].into_iter().collect()));
        ev.propagate_derived("m1", "Math").unwrap();
        let inst = ev.store.get("m1").unwrap();
        assert_eq!(inst.get("res").unwrap(), &Value::Number(20.0));
    }

    #[test]
    fn test_arithmetic() {
        let ev = empty_eval();
        let expr = Expr::Binary {
            op: BinOp::Mul,
            left: Box::new(Expr::Binary {
                op: BinOp::Add,
                left: Box::new(Expr::Number(2.0)),
                right: Box::new(Expr::Number(3.0)),
                span: Span::default(),
            }),
            right: Box::new(Expr::Number(4.0)),
            span: Span::default(),
        };
        assert_eq!(ev.eval_expr(&expr, None).unwrap(), Value::Number(20.0));
    }

    #[test]
    fn test_if_then_else() {
        let ev = empty_eval();
        let expr = Expr::If {
            cond: Box::new(Expr::Bool(true)),
            then_: Box::new(Expr::Number(1.0)),
            else_: Box::new(Expr::Number(2.0)),
            span: Span::default(),
        };
        assert_eq!(ev.eval_expr(&expr, None).unwrap(), Value::Number(1.0));
    }

    #[test]
    fn test_interpolation() {
        let mut ev = empty_eval();
        ev.env.insert("name".into(), Value::Text("Isaac".into()));
        ev.env.insert("age".into(), Value::Number(26.0));
        let expr = Expr::InterpolatedString(vec![
            StringSegment::Literal("Hello ".into()),
            StringSegment::Expr(Box::new(Expr::Ident("name".into()))),
            StringSegment::Literal(", you are ".into()),
            StringSegment::Expr(Box::new(Expr::Ident("age".into()))),
            StringSegment::Literal(" years old".into()),
        ]);
        assert_eq!(
            ev.eval_expr(&expr, None).unwrap(),
            Value::Text("Hello Isaac, you are 26 years old".into())
        );
    }

    #[test]
    fn test_derived_recomputes() {
        let mut ev = build_eval("entity Person {\n  age: Number\n  isAdult := age >= 18\n}");
        let mut fields = HashMap::new();
        fields.insert("age".into(), Value::Number(17.0));
        fields.insert("isAdult".into(), Value::Bool(false));
        ev.store.insert("p1", Instance::new("Person", fields));

        ev.apply_update("p1", "age", Value::Number(18.0)).unwrap();
        assert_eq!(ev.store.get("p1").unwrap().get("isAdult"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_rule_fires_on_becomes() {
        let src = "entity S {\n  active: Boolean\n}\nrule \"activate\" {\n  when S.active becomes true\n  then show \"fired\"\n}";
        let mut ev = build_eval(src);
        let mut fields = HashMap::new();
        fields.insert("active".into(), Value::Bool(false));
        ev.store.insert("S", Instance::new("S", fields));

        let events = ev.apply_update("S", "active", Value::Bool(true)).unwrap();
        assert!(events.iter().any(|e| e.rule == "activate"));
    }

    #[test]
    fn test_rollback_on_div_zero() {
        let mut ev = build_eval("entity A {\n  x: Number\n  y: Number\n  ratio := x / y\n}");
        let mut fields = HashMap::new();
        fields.insert("x".into(), Value::Number(10.0));
        fields.insert("y".into(), Value::Number(2.0));
        fields.insert("ratio".into(), Value::Number(5.0));
        ev.store.insert("a1", Instance::new("A", fields));

        let result = ev.apply_update("a1", "y", Value::Number(0.0));
        assert!(result.is_err());
        // Store should be rolled back
        assert_eq!(ev.store.get("a1").unwrap().get("y"), Some(&Value::Number(2.0)));
    }

    #[test]
    fn test_export_state() {
        let mut ev = empty_eval();
        let mut fields = HashMap::new();
        fields.insert("name".into(), Value::Text("Isaac".into()));
        fields.insert("age".into(), Value::Number(26.0));
        ev.store.insert("isaac", Instance::new("Person", fields));

        let state = ev.export_state();
        assert!(state["instances"]["isaac"]["entity"] == "Person");
        assert!(state["instances"]["isaac"]["fields"]["name"] == "Isaac");
        assert!(state["instances"]["isaac"]["fields"]["age"] == 26);
        assert!(state["stable"] == true);
    }

    #[test]
    fn test_rule_does_not_fire_without_transition() {
        let src = "entity S {\n  active: Boolean\n}\nrule \"activate\" {\n  when S.active becomes true\n  then show \"fired\"\n}";
        let mut ev = build_eval(src);
        let mut fields = HashMap::new();
        fields.insert("active".into(), Value::Bool(true));
        ev.store.insert("S", Instance::new("S", fields));
        // Commit so prev_fields = fields (active=true already)
        ev.store.commit_all();

        let events = ev.apply_update("S", "active", Value::Bool(true)).unwrap();
        assert!(events.iter().all(|e| e.rule != "activate"));
    }

    #[test]
    fn test_adapter_poll_triggers_rule() {
        // Guide §28.5 Step 8: external entity + StaticAdapter, push value, verify rule fires
        let src = "entity Sensor {\n  reading: Number\n  isCritical := reading > 90\n}\nrule \"overheat\" {\n  when Sensor.isCritical becomes true\n  then show \"overheating\"\n}";
        let mut ev = build_eval(src);
        let mut fields = HashMap::new();
        fields.insert("reading".into(), Value::Number(50.0));
        fields.insert("isCritical".into(), Value::Bool(false));
        ev.store.insert("Sensor", Instance::new("Sensor", fields));

        // Register a StaticAdapter and push a critical reading
        let mut adapter = crate::adapters::static_adapter::StaticAdapter::new("Sensor");
        adapter.push("reading", Value::Number(95.0));
        ev.register_adapter(Box::new(adapter));

        // tick() should poll the adapter and fire the overheat rule
        let result = ev.tick();
        assert!(result.is_ok());
        // After tick, reading should be updated
        assert_eq!(
            ev.store.get("Sensor").unwrap().get("reading"),
            Some(&Value::Number(95.0))
        );
        // isCritical should have been recomputed
        assert_eq!(
            ev.store.get("Sensor").unwrap().get("isCritical"),
            Some(&Value::Bool(true))
        );
    }

    #[test]
    fn test_unregistered_entity_ignored() {
        // Guide §28.5 Step 9: entities without a registered adapter are silently ignored
        let src = "entity Sensor {\n  reading: Number\n}";
        let mut ev = build_eval(src);

        // Register adapter for an entity that has no instance in the store
        let mut adapter = crate::adapters::static_adapter::StaticAdapter::new("UnknownEntity");
        adapter.push("value", Value::Number(42.0));
        ev.register_adapter(Box::new(adapter));

        // tick() should not panic or error
        let result = ev.tick();
        assert!(result.is_ok());
    }

    #[test]
    fn test_prev_value_access() {
        let src = r#"
entity Battery {
  level: Number
  drop := prev(level) - level
}
        "#;
        let mut ev = build_eval(src);
        
        let mut fields = HashMap::new();
        fields.insert("level".into(), Value::Number(100.0));
        fields.insert("drop".into(), Value::Number(0.0)); // Initial
        
        ev.store.insert("batt1", Instance::new("Battery", fields));
        ev.store.commit_all(); // Commit baseline state
        
        // Update level to 90
        ev.apply_update("batt1", "level", Value::Number(90.0)).unwrap();
        
        // Check derived drop
        assert_eq!(
            ev.store.get("batt1").unwrap().get("drop"),
            Some(&Value::Number(10.0)) // 100 - 90
        );
    }

    #[test]
    fn test_cascading_cleanup_issue_6() {
        let source = r#"
            entity Resource { status: Text }
            rule "cleanup" {
                when Resource.status == "deleted" becomes true
                then delete Resource
            }
            rule "log_deleted" {
                when Resource.status == "deleted" becomes true
                then update Resource.status to "forgotten"
            }
        "#;
        let mut ev = build_eval(source);
        ev.instances.insert("res1".to_string(), "Resource".to_string());
        ev.store.insert("res1".to_string(), crate::store::Instance::new("Resource", vec![("status".to_string(), Value::Text("active".to_string()))].into_iter().collect()));

        
        let res = ev.apply_update("res1", "status", Value::Text("deleted".to_string()));
        
        // Output should not panic or return an error like R001 "Unknown instance" because the second rule will be skipped safely
        res.unwrap();
        // Specifically, it should not exist in the store
        assert!(ev.store.get("res1").is_none());
    }
}
