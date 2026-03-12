use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::collections::HashMap;
use std::cell::RefCell;
use lumina_parser::parse;
use lumina_parser::ast::*;
use lumina_analyzer::analyze;
use lumina_runtime::engine::Evaluator;
use lumina_runtime::value::Value;

thread_local! {
    static LAST_CREATE_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

/// Opaque runtime handle — the caller never sees internals
pub struct LuminaRuntime {
    evaluator: Evaluator,
    last_error: Option<String>,
}

fn to_c_string(s: &str) -> *mut c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

fn build_evaluator(analyzed: &lumina_analyzer::AnalyzedProgram) -> Evaluator {
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
    let mut ev = Evaluator::new(analyzed.schema.clone(), analyzed.graph.clone(), rules);
    ev.derived_exprs = derived;
    ev
}

fn json_to_value(jv: &serde_json::Value) -> Option<Value> {
    match jv {
        serde_json::Value::Number(n) => n.as_f64().map(Value::Number),
        serde_json::Value::String(s) => Some(Value::Text(s.clone())),
        serde_json::Value::Bool(b) => Some(Value::Bool(*b)),
        _ => None,
    }
}

// ── C API ──────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn lumina_create(source: *const c_char) -> *mut LuminaRuntime {
    if source.is_null() { return std::ptr::null_mut(); }

    let src = unsafe { CStr::from_ptr(source) };
    let src = match src.to_str() {
        Ok(s) => s,
        Err(e) => {
            LAST_CREATE_ERROR.with(|cell| *cell.borrow_mut() = Some(format!("Invalid UTF-8: {e}")));
            return std::ptr::null_mut();
        }
    };

    let program = match parse(src) {
        Ok(p) => p,
        Err(e) => {
            LAST_CREATE_ERROR.with(|cell| *cell.borrow_mut() = Some(format!("Parse error: {e}")));
            return std::ptr::null_mut();
        }
    };

    let analyzed = match analyze(program, src, "<FFI>") {
        Ok(a) => a,
        Err(errors) => {
            let msg = errors.iter()
                .map(|e| format!("[{}] {}", e.code, e.message))
                .collect::<Vec<_>>()
                .join("\n");
            LAST_CREATE_ERROR.with(|cell| *cell.borrow_mut() = Some(msg));
            return std::ptr::null_mut();
        }
    };

    let mut evaluator = build_evaluator(&analyzed);
    for stmt in &analyzed.program.statements {
        if let Err(e) = evaluator.exec_statement(stmt) {
            LAST_CREATE_ERROR.with(|cell| {
                *cell.borrow_mut() = Some(format!("[{}] {}", e.code(), e.message()));
            });
            return std::ptr::null_mut();
        }
    }

    Box::into_raw(Box::new(LuminaRuntime { evaluator, last_error: None }))
}

#[no_mangle]
pub extern "C" fn lumina_apply_event(
    runtime:       *mut LuminaRuntime,
    instance_name: *const c_char,
    field_name:    *const c_char,
    value_json:    *const c_char,
) -> *mut c_char {
    if runtime.is_null() || instance_name.is_null()
        || field_name.is_null() || value_json.is_null()
    {
        return std::ptr::null_mut();
    }

    let rt = unsafe { &mut *runtime };
    let inst = unsafe { CStr::from_ptr(instance_name) }.to_string_lossy();
    let field = unsafe { CStr::from_ptr(field_name) }.to_string_lossy();
    let val_str = unsafe { CStr::from_ptr(value_json) }.to_string_lossy();

    let json_val: serde_json::Value = match serde_json::from_str(&val_str) {
        Ok(v) => v,
        Err(e) => {
            rt.last_error = Some(format!("Invalid JSON value: {e}"));
            return std::ptr::null_mut();
        }
    };

    let value = match json_to_value(&json_val) {
        Some(v) => v,
        None => {
            rt.last_error = Some("Unsupported value type".to_string());
            return std::ptr::null_mut();
        }
    };

    match rt.evaluator.apply_event(&inst, &field, value) {
        Ok(prop) => {
            rt.last_error = None;
            to_c_string(&serde_json::to_string(&prop).unwrap_or_default())
        }
        Err(rollback) => {
            let diag_json = serde_json::to_string(&rollback.diagnostic).unwrap_or_default();
            rt.last_error = Some(rollback.diagnostic.message.clone());
            to_c_string(&format!("ERROR:{diag_json}"))
        }
    }
}

#[no_mangle]
pub extern "C" fn lumina_export_state(runtime: *const LuminaRuntime) -> *mut c_char {
    if runtime.is_null() { return std::ptr::null_mut(); }
    let rt = unsafe { &*runtime };
    let state = rt.evaluator.export_state();
    to_c_string(&serde_json::to_string_pretty(&state).unwrap_or_default())
}

#[no_mangle]
pub extern "C" fn lumina_tick(runtime: *mut LuminaRuntime) -> *mut c_char {
    if runtime.is_null() { return std::ptr::null_mut(); }
    let rt = unsafe { &mut *runtime };
    match rt.evaluator.tick() {
        Ok(events) => {
            let arr: Vec<serde_json::Value> = events.iter().map(|e| {
                serde_json::json!({"rule": e.rule, "instance": e.instance})
            }).collect();
            to_c_string(&serde_json::to_string(&arr).unwrap_or_default())
        }
        Err(rollback) => {
            let diag_json = serde_json::to_string(&rollback.diagnostic).unwrap_or_default();
            rt.last_error = Some(rollback.diagnostic.message.clone());
            to_c_string(&format!("ERROR:{diag_json}"))
        }
    }
}

#[no_mangle]
pub extern "C" fn lumina_last_error(runtime: *const LuminaRuntime) -> *mut c_char {
    if runtime.is_null() {
        // Check thread-local for creation errors
        return LAST_CREATE_ERROR.with(|cell| {
            match cell.borrow().as_ref() {
                Some(msg) => to_c_string(msg),
                None => to_c_string(""),
            }
        });
    }
    let rt = unsafe { &*runtime };
    match &rt.last_error {
        Some(msg) => to_c_string(msg),
        None => to_c_string(""),
    }
}

#[no_mangle]
pub extern "C" fn lumina_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)); }
    }
}

#[no_mangle]
pub extern "C" fn lumina_destroy(runtime: *mut LuminaRuntime) {
    if !runtime.is_null() {
        unsafe { drop(Box::from_raw(runtime)); }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_source() -> CString {
        CString::new(concat!(
            "entity Sensor {\n",
            "  temp: Number\n",
            "  isHot := temp > 30\n",
            "}\n",
            "let Sensor = Sensor { temp: 25 }\n",
        )).unwrap()
    }

    #[test]
    fn test_create_and_export() {
        let src = test_source();
        let rt = lumina_create(src.as_ptr());
        assert!(!rt.is_null());

        let state_ptr = lumina_export_state(rt);
        assert!(!state_ptr.is_null());
        let state = unsafe { CStr::from_ptr(state_ptr) }.to_string_lossy().to_string();
        assert!(state.contains("Sensor"));
        assert!(state.contains("temp"));

        lumina_free_string(state_ptr);
        lumina_destroy(rt);
    }

    #[test]
    fn test_apply_event_success() {
        let src = test_source();
        let rt = lumina_create(src.as_ptr());
        assert!(!rt.is_null());

        let inst = CString::new("Sensor").unwrap();
        let field = CString::new("temp").unwrap();
        let val = CString::new("35").unwrap();

        let result_ptr = lumina_apply_event(rt, inst.as_ptr(), field.as_ptr(), val.as_ptr());
        assert!(!result_ptr.is_null());
        let result = unsafe { CStr::from_ptr(result_ptr) }.to_string_lossy().to_string();
        assert!(result.contains("success"));
        assert!(!result.starts_with("ERROR:"));

        // Check state reflects the update
        let state_ptr = lumina_export_state(rt);
        let state = unsafe { CStr::from_ptr(state_ptr) }.to_string_lossy().to_string();
        assert!(state.contains("true")); // isHot should be true now

        lumina_free_string(result_ptr);
        lumina_free_string(state_ptr);
        lumina_destroy(rt);
    }

    #[test]
    fn test_apply_event_derived_field_error() {
        let src = test_source();
        let rt = lumina_create(src.as_ptr());
        assert!(!rt.is_null());

        let inst = CString::new("Sensor").unwrap();
        let field = CString::new("isHot").unwrap();
        let val = CString::new("true").unwrap();

        let result_ptr = lumina_apply_event(rt, inst.as_ptr(), field.as_ptr(), val.as_ptr());
        assert!(!result_ptr.is_null());
        let result = unsafe { CStr::from_ptr(result_ptr) }.to_string_lossy().to_string();
        assert!(result.starts_with("ERROR:"), "expected ERROR, got: {result}");

        lumina_free_string(result_ptr);
        lumina_destroy(rt);
    }

    #[test]
    fn test_create_invalid_source() {
        let bad = CString::new("entity { broken").unwrap();
        let rt = lumina_create(bad.as_ptr());
        assert!(rt.is_null());

        let err_ptr = lumina_last_error(std::ptr::null());
        assert!(!err_ptr.is_null());
        let err = unsafe { CStr::from_ptr(err_ptr) }.to_string_lossy().to_string();
        assert!(!err.is_empty());

        lumina_free_string(err_ptr);
    }

    #[test]
    fn test_null_safety() {
        lumina_destroy(std::ptr::null_mut());
        lumina_free_string(std::ptr::null_mut());
        assert!(lumina_export_state(std::ptr::null()).is_null());
        assert!(lumina_apply_event(
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        ).is_null());
    }
}
