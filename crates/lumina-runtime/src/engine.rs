use std::collections::{HashMap, HashSet};
use lumina_analyzer::types::Schema;
use lumina_analyzer::graph::DependencyGraph;
use lumina_parser::ast::*;
use crate::value::Value;
use crate::store::{EntityStore, Instance};
use crate::snapshot::{SnapshotStack, PropResult, FiredEvent, RollbackResult, Diagnostic};
use crate::RuntimeError;
use crate::rules;
use crate::timers::TimerHeap;

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
    pub timers:    TimerHeap,
    depth:         usize,
    fired_this_cycle: HashSet<String>,
    output:        Vec<String>,
}

impl Evaluator {
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
            timers,
            depth: 0,
            fired_this_cycle: HashSet::new(),
            output: Vec::new(),
        }
    }

    pub fn register_derived(&mut self, entity: &str, field: &str, expr: Expr) {
        self.derived_exprs.insert((entity.to_string(), field.to_string()), expr);
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
                Err(RuntimeError::R001 { instance: name.clone() })
            }

            Expr::FieldAccess { obj, field, .. } => {
                let inst_name = match obj.as_ref() {
                    Expr::Ident(n) => n.clone(),
                    _ => return Err(RuntimeError::R001 { instance: format!("{:?}", obj) }),
                };
                let instance = self.store.get(&inst_name)
                    .ok_or(RuntimeError::R001 { instance: inst_name.clone() })?;
                instance.get(field).cloned()
                    .ok_or(RuntimeError::R005 { instance: inst_name, field: field.clone() })
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

            Expr::Interpolated { segments, .. } => {
                let mut out = String::new();
                for seg in segments {
                    match seg {
                        Segment::Literal(s) => out.push_str(s),
                        Segment::Expr(e) => {
                            let v = self.eval_expr(e, ctx)?;
                            out.push_str(&v.to_string());
                        }
                    }
                }
                Ok(Value::Text(out))
            }
        }
    }

    // ── Statement executor ────────────────────────────────

    pub fn exec_statement(&mut self, stmt: &Statement) -> Result<(), RuntimeError> {
        match stmt {
            Statement::Entity(_) | Statement::ExternalEntity(_) | Statement::Rule(_) => Ok(()),
            Statement::Let(ls) => {
                match &ls.value {
                    LetValue::Expr(expr) => {
                        let val = self.eval_expr(expr, None)?;
                        self.env.insert(ls.name.clone(), val);
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
                    }
                }
                Ok(())
            }
            Statement::Action(a) => { self.exec_action(a)?; Ok(()) }
        }
    }

    // ── Action executor ───────────────────────────────────

    pub fn exec_action(&mut self, action: &Action) -> Result<Vec<FiredEvent>, RuntimeError> {
        match action {
            Action::Show(expr) => {
                let val = self.eval_expr(expr, None)?;
                let s = val.to_string();
                println!("{}", s);
                self.output.push(s);
                Ok(vec![])
            }
            Action::Update { target, value } => {
                let val = self.eval_expr(value, None)?;
                self.apply_update(&target.instance, &target.field, val)
            }
            Action::Create { entity, fields } => {
                let mut fv = HashMap::new();
                for (name, expr) in fields {
                    fv.insert(name.clone(), self.eval_expr(expr, None)?);
                }
                let count = self.store.all_of_entity(entity).count();
                let inst_name = format!("{}_{}", entity.to_lowercase(), count + 1);
                self.instances.insert(inst_name.clone(), entity.clone());
                self.store.insert(inst_name, Instance::new(entity, fv));
                Ok(vec![])
            }
            Action::Delete(name) => {
                self.store.remove(name).ok_or(RuntimeError::R001 { instance: name.clone() })?;
                Ok(vec![])
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

        let snap = self.snapshots.take(&self.store);
        self.snapshots.push(snap);

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

        // Apply
        self.store.get_mut(instance_name)
            .ok_or(RuntimeError::R001 { instance: instance_name.to_string() })?
            .set(field_name, new_value);

        // Propagate derived fields
        if let Err(e) = self.propagate_derived(instance_name, &entity_name) {
            let snap = self.snapshots.pop().unwrap();
            self.store = snap.store;
            self.depth -= 1;
            return Err(e);
        }

        // Evaluate rules
        let mut all_events = Vec::new();
        let rules_clone = self.rules.clone();
        for rule in &rules_clone {
            if let RuleTrigger::When(condition) = &rule.trigger {
                match rules::condition_is_met(self, condition, instance_name) {
                    Ok(true) => {
                        let fire_key = format!("{}::{}", rule.name, instance_name);
                        if self.fired_this_cycle.contains(&fire_key) {
                            // Already fired in this cycle, skip
                            continue;
                        }
                        if let Some(dur) = &condition.for_duration {
                            let _ = self.timers.start_for_timer(
                                &rule.name, instance_name, dur.to_seconds(),
                            );
                        } else {
                            self.fired_this_cycle.insert(fire_key);
                            for action in &rule.actions {
                                match self.exec_action(action) {
                                    Ok(evts) => all_events.extend(evts),
                                    Err(e) => {
                                        let snap = self.snapshots.pop().unwrap();
                                        self.store = snap.store;
                                        self.depth -= 1;
                                        return Err(e);
                                    }
                                }
                            }
                            all_events.push(FiredEvent {
                                rule: rule.name.clone(),
                                instance: instance_name.to_string(),
                            });
                        }
                    }
                    Ok(false) => {
                        // Condition not met — cancel any pending for-timer
                        self.timers.cancel_for_timer(&rule.name, instance_name);
                    }
                    Err(e) => {
                        let snap = self.snapshots.pop().unwrap();
                        self.store = snap.store;
                        self.depth -= 1;
                        return Err(e);
                    }
                }
            }
        }

        // Only commit at outermost level to prevent re-triggering becomes
        if self.depth == 1 {
            self.store.commit_all();
            self.fired_this_cycle.clear();
        }
        self.snapshots.pop();
        self.depth -= 1;
        Ok(all_events)
    }

    fn propagate_derived(&mut self, instance_name: &str, entity_name: &str) -> Result<(), RuntimeError> {
        let mut derived: Vec<(String, String)> = self.derived_exprs.keys()
            .filter(|(ent, _)| ent == entity_name)
            .cloned()
            .collect();
        derived.sort_by_key(|(e, f)| self.graph.get_node(e, f).unwrap_or(u32::MAX));

        for (ent, field) in derived {
            if let Some(expr) = self.derived_exprs.get(&(ent, field.clone())).cloned() {
                let val = self.eval_expr(&expr, Some(instance_name))?;
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
        let mut events = vec![];
        for action in &rule.actions {
            let evts = self.exec_action(action)?;
            events.extend(evts);
        }
        events.push(FiredEvent {
            rule: rule.name.clone(),
            instance: instance_name.to_string(),
        });
        Ok(events)
    }

    /// Called periodically by the host — fires any elapsed for/every timers
    pub fn tick(&mut self) -> Result<Vec<FiredEvent>, RollbackResult> {
        let mut all_events = vec![];

        // ── Fire elapsed `for` timers ──────────────────────────────────
        let elapsed = self.timers.drain_elapsed_for_timers();
        for timer in elapsed {
            let rule = self.rules.iter()
                .find(|r| r.name == timer.rule_name)
                .cloned();
            if let Some(rule) = rule {
                if let RuleTrigger::When(condition) = &rule.trigger {
                    let still_true = rules::condition_is_met(
                        self, condition, &timer.instance_name
                    ).unwrap_or(false);
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
                fields.insert(fname.clone(), match val {
                    Value::Number(n) if n.fract() == 0.0 => serde_json::json!(*n as i64),
                    Value::Number(n) => serde_json::json!(*n),
                    Value::Text(s) => serde_json::json!(s),
                    Value::Bool(b) => serde_json::json!(b),
                });
            }
            instances.insert(name.clone(), serde_json::json!({
                "entity": instance.entity_name,
                "fields": fields,
            }));
        }
        serde_json::json!({
            "instances": instances,
            "stable": true,
            "version": self.snapshots.current_version()
        })
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
        let analyzed = lumina_analyzer::analyze(program, source, "<runtime-test>").expect("analysis failed");
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
        ev
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
        let expr = Expr::Interpolated {
            segments: vec![
                Segment::Literal("Hello ".into()),
                Segment::Expr(Expr::Ident("name".into())),
                Segment::Literal(", you are ".into()),
                Segment::Expr(Expr::Ident("age".into())),
                Segment::Literal(" years old".into()),
            ],
            span: Span::default(),
        };
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
}
