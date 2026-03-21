use std::collections::HashMap;
use crate::value::Value;
use crate::store::EntityStore;
use lumina_parser::ast::{AggregateDecl, AggregateExpr};

pub struct AggregateStore {
    decls:  Vec<AggregateDecl>,
    values: HashMap<String, HashMap<String, Value>>,
}

impl AggregateStore {
    pub fn new() -> Self {
        Self { decls: Vec::new(), values: HashMap::new() }
    }

    pub fn register(&mut self, decl: AggregateDecl) {
        self.decls.push(decl);
    }

    pub fn get(&self, agg: &str, field: &str) -> Option<&Value> {
        self.values.get(agg)?.get(field)
    }

    pub fn recompute(&mut self, store: &EntityStore) {
        for decl in &self.decls {
            let instances: Vec<String> = store
                .all_of_entity(&decl.over)
                .map(|(n, _)| n.clone())
                .collect();
            let mut agg_vals = HashMap::new();
            for field in &decl.fields {
                let val = compute_agg(&field.expr, &instances, store);
                agg_vals.insert(field.name.clone(), val);
            }
            self.values.insert(decl.name.clone(), agg_vals);
        }
    }
}

fn nums(insts: &[String], field: &str, store: &EntityStore) -> Vec<f64> {
    insts.iter().filter_map(|i| {
        store.get(i)?.get(field).and_then(|v| {
            if let Value::Number(n) = v { Some(*n) } else { None }
        })
    }).collect()
}

fn bools(insts: &[String], field: &str, store: &EntityStore) -> Vec<bool> {
    insts.iter().filter_map(|i| {
        store.get(i)?.get(field).and_then(|v| {
            if let Value::Bool(b) = v { Some(*b) } else { None }
        })
    }).collect()
}

fn compute_agg(
    expr: &AggregateExpr,
    insts: &[String],
    store: &EntityStore,
) -> Value {
    match expr {
        AggregateExpr::Avg(f) => {
            let ns = nums(insts, f, store);
            if ns.is_empty() { return Value::Number(0.0); }
            Value::Number(ns.iter().sum::<f64>() / ns.len() as f64)
        }
        AggregateExpr::Min(f) => {
            let ns = nums(insts, f, store);
            Value::Number(ns.into_iter().fold(f64::INFINITY, f64::min))
        }
        AggregateExpr::Max(f) => {
            let ns = nums(insts, f, store);
            Value::Number(ns.into_iter().fold(f64::NEG_INFINITY, f64::max))
        }
        AggregateExpr::Sum(f) => {
            Value::Number(nums(insts, f, store).iter().sum())
        }
        AggregateExpr::Count(None) => {
            Value::Number(insts.len() as f64)
        }
        AggregateExpr::Count(Some(f)) => {
            Value::Number(
                bools(insts, f, store).iter().filter(|&&b| b).count() as f64
            )
        }
        AggregateExpr::Any(f) => {
            Value::Bool(bools(insts, f, store).iter().any(|&b| b))
        }
        AggregateExpr::All(f) => {
            Value::Bool(
                !insts.is_empty() &&
                bools(insts, f, store).iter().all(|&b| b)
            )
        }
    }
}
