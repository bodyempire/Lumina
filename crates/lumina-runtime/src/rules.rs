use lumina_parser::ast::*;
use crate::engine::Evaluator;
use crate::value::Value;
use crate::RuntimeError;

pub fn condition_is_met(
    evaluator: &Evaluator,
    condition: &Condition,
    instance_name: &str,
) -> Result<bool, RuntimeError> {
    let current = evaluator.eval_expr(&condition.expr, Some(instance_name))?;

    match &condition.becomes {
        None => Ok(current == Value::Bool(true)),
        Some(target_expr) => {
            let target = evaluator.eval_expr(target_expr, Some(instance_name))?;
            if current != target {
                return Ok(false);
            }
            // Transition check: at least one field must have changed since last commit
            if let Some(instance) = evaluator.store.get(instance_name) {
                let has_transition = instance.fields.iter().any(|(k, v)| {
                    instance.prev_fields.get(k) != Some(v)
                });
                Ok(has_transition)
            } else {
                Ok(false)
            }
        }
    }
}
