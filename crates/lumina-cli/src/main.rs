use std::fs;
use std::collections::HashMap;
mod repl;
mod commands;

use lumina_parser::parse;
use lumina_parser::ast::*;
use lumina_analyzer::analyze;
use lumina_runtime::engine::Evaluator;
use lumina_diagnostics::DiagnosticRenderer;

mod loader;
use crate::loader::ModuleLoader;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("run")   => cmd_run(&args),
        Some("check") => cmd_check(&args),
        Some("repl")  => cmd_repl(),
        _ => {
            eprintln!("Lumina v1.5 — Declarative Reactive Language");
            eprintln!();
            eprintln!("Usage:");
            eprintln!("  lumina run <file.lum>     Run a Lumina program");
            eprintln!("  lumina check <file.lum>   Type-check without running");
            eprintln!("  lumina repl               Start interactive REPL");
            std::process::exit(1);
        }
    }
}

fn read_file(args: &[String]) -> (String, String) {
    let path = args.get(2).unwrap_or_else(|| {
        eprintln!("Error: missing file argument");
        eprintln!("Usage: lumina run <file.lum>");
        std::process::exit(1);
    });
    let source = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading file '{path}': {e}");
        std::process::exit(1);
    });
    (path.clone(), source)
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

fn cmd_run(args: &[String]) {
    let (path, source) = read_file(args);

    let program = match ModuleLoader::load(Path::new(&path)) {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };

    let analyzed = analyze(program, &source, &path, true).unwrap_or_else(|errors| {
        eprintln!("{}", DiagnosticRenderer::render_all(&errors));
        std::process::exit(1);
    });

    let mut evaluator = build_evaluator(&analyzed);

    for stmt in &analyzed.program.statements {
        if let Err(e) = evaluator.exec_statement(stmt) {
            eprintln!("Runtime error [{}]: {}", e.code(), e.message());
            std::process::exit(1);
        }
    }

    // Initial state calculation
    if let Err(e) = evaluator.recalculate_all_rules() {
        eprintln!("Initialization error: {}", e.message());
        std::process::exit(1);
    }

    if evaluator.timers.for_timers.is_empty() 
        && evaluator.timers.every_timers.is_empty() 
        && evaluator.adapters.is_empty() 
    {
        return;
    }

    println!("Running Lumina [Ctrl+C to stop]...");
    loop {
        if let Err(rollback) = evaluator.tick() {
            eprintln!("Runtime error: {}", rollback.diagnostic.message);
            std::process::exit(1);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn cmd_check(args: &[String]) {
    let (path, source) = read_file(args);

    let program = match ModuleLoader::load(Path::new(&path)) {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };

    match analyze(program, &source, &path, true) {
        Ok(_) => {
            let basename = std::path::Path::new(&path)
                .file_name().unwrap_or_default()
                .to_string_lossy();
            println!("✓ {} — no errors found", basename);
        }
        Err(errors) => {
            eprintln!("{}", DiagnosticRenderer::render_all(&errors));
            std::process::exit(1);
        }
    }
}

fn cmd_repl() {
    use crate::repl::{ReplSession, ReplResult};
    use crate::commands::run_command;
    use std::io::{self, BufRead, Write};

    println!("Lumina v1.5 REPL — type Lumina expressions and statements");
    println!("Type ':help' to see inspector commands\n");

    let mut session = ReplSession::new();
    let stdin = io::stdin();

    loop {
        // Show prompt based on brace depth
        let prompt = if session.brace_depth > 0 { "... " } else { "lumina> " };
        print!("{}", prompt);
        io::stdout().flush().ok();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 { break; }
        let line = line.trim_end_matches('\n').trim_end_matches('\r');

        // Inspector commands start with ":"
        if line.starts_with(':') {
            println!("{}", run_command(&mut session, line));
            continue;
        }

        // Skip blank lines
        if line.trim().is_empty() { continue; }

        match session.feed(line) {
            ReplResult::NeedMore => {} // show "..." next iteration
            ReplResult::Ok(out) => { if !out.is_empty() { println!("{}", out); } }
            ReplResult::Error(err) => { eprintln!("{}", err); }
        }
    }
}
