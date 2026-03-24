use lumina_parser::parse;
use lumina_analyzer::analyze;
use lumina_runtime::engine::Evaluator;
use lumina_diagnostics::DiagnosticRenderer;
use lumina_parser::ast::{Statement, Field};

pub struct ReplSession {
    pub evaluator: Evaluator,
    source_accum: String,
    pub brace_depth: i32,
    history: Vec<String>,
    /// Accumulated source across ALL inputs - used by :save
    pub full_history: String,
}

pub enum ReplResult {
    NeedMore, // multi-line construct - show "..." prompt
    Ok(String), // success - optional output string to print
    Error(String), // error - rendered diagnostic string
}

impl ReplSession {
    pub fn new() -> Self {
        Self {
            evaluator: Evaluator::new_empty(), // see note below
            source_accum: String::new(),
            brace_depth: 0,
            history: Vec::new(),
            full_history: String::new(),
        }
    }

    /// Feed one line of input. Returns what the REPL loop should do.
    pub fn feed(&mut self, line: &str) -> ReplResult {
        // Track brace depth for multi-line detection
        for ch in line.chars() {
            match ch {
                '{' => self.brace_depth += 1,
                '}' => self.brace_depth -= 1,
                _=> {}
            }
        }

        self.source_accum.push_str(line);
        self.source_accum.push('\n');

        // Multi-line construct still open
        if self.brace_depth > 0 { return ReplResult::NeedMore; }

        // Complete construct - drain accumulator and execute
        let source = std::mem::take(&mut self.source_accum);
        self.history.push(source.clone());
        self.full_history.push_str(&source);

        self.exec_source(&source)
    }

    fn exec_source(&mut self, source: &str) -> ReplResult {
        let program = match parse(source) {
            Ok(p) => p,
            Err(e) => return ReplResult::Error(format!("parse error: {}", e)),
        };

        // Note: the REPL in v1.4 evaluates statements progressively, but the analyzer needs the full program context.
        // For the REPL, we analyze the current snippet. Realistically it needs full history, but for simplicity
        // based on the spec, we pass `&program` to `analyze` here.
        let full_program = match parse(&self.full_history) {
            Ok(p) => p,
            Err(e) => return ReplResult::Error(format!("full history parse error: {}", e)),
        };

        let analyzed = match analyze(full_program, &self.full_history, "<repl>", true) {
            Ok(a) => a,
            Err(diags) => return ReplResult::Error(DiagnosticRenderer::render_all(&diags)),
        };

        // Update schema and graph incrementally from the analyzed full history
        self.evaluator.schema = analyzed.schema;
        self.evaluator.graph = analyzed.graph;

        let mut output = Vec::new();

        // Only explicitly execute the new statements
        for stmt in &program.statements {
            // Also need to register any new derived exprs
            if let Statement::Entity(e) = stmt {
                for f in &e.fields {
                    if let Field::Derived(df) = f {
                        self.evaluator.register_derived(&e.name, &df.name, df.expr.clone());
                    }
                }
            }
            if let Statement::Rule(r) = stmt {
                self.evaluator.rules.push(r.clone());
            }

            match self.evaluator.exec_statement(stmt) {
                Ok(_) => {}
                Err(e) => return ReplResult::Error(format!("{:?}", e)),
            }
        }

        // Collect any show output from WASM-style buffer (if enabled)
        let captured = self.evaluator.drain_output();
        output.extend(captured);

        ReplResult::Ok(output.join("\n"))
    }

    /// Reset to a fresh session.
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}
