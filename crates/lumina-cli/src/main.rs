use std::fs;
use std::collections::HashMap;
use lumina_parser::parse;
use lumina_parser::ast::*;
use lumina_analyzer::analyze;
use lumina_runtime::engine::Evaluator;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("run")   => cmd_run(&args),
        Some("check") => cmd_check(&args),
        Some("repl")  => cmd_repl(),
        _ => {
            eprintln!("Lumina v1.3 — Declarative Reactive Language");
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
    let (_, source) = read_file(args);

    let program = parse(&source).unwrap_or_else(|e| {
        eprintln!("Parse error: {e}");
        std::process::exit(1);
    });

    let analyzed = analyze(program).unwrap_or_else(|errors| {
        for e in &errors {
            eprintln!("[{}] {} (line {})", e.code, e.message, e.span.line);
        }
        std::process::exit(1);
    });

    let mut evaluator = build_evaluator(&analyzed);

    for stmt in &analyzed.program.statements {
        if let Err(e) = evaluator.exec_statement(stmt) {
            eprintln!("Runtime error [{}]: {}", e.code(), e.message());
            std::process::exit(1);
        }
    }

    let state = evaluator.export_state();
    println!("{}", serde_json::to_string_pretty(&state).unwrap());
}

fn cmd_check(args: &[String]) {
    let (path, source) = read_file(args);

    let program = parse(&source).unwrap_or_else(|e| {
        eprintln!("Parse error: {e}");
        std::process::exit(1);
    });

    match analyze(program) {
        Ok(_) => {
            let basename = std::path::Path::new(&path)
                .file_name().unwrap_or_default()
                .to_string_lossy();
            println!("✓ {} — no errors found", basename);
        }
        Err(errors) => {
            for e in &errors {
                eprintln!("[{}] {} (line {})", e.code, e.message, e.span.line);
            }
            std::process::exit(1);
        }
    }
}

fn cmd_repl() {
    use std::io::{self, BufRead, Write};

    println!("Lumina v1.3 REPL — type Lumina expressions and statements");
    println!("Type 'exit' to quit, 'state' to see current state\n");

    let stdin = io::stdin();
    let mut accumulated_source = String::new();
    let mut evaluator: Option<Evaluator> = None;

    loop {
        print!("lumina> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        stdin.lock().read_line(&mut line).unwrap();
        let line = line.trim();

        match line {
            "exit" | "quit" => break,
            "state" => {
                if let Some(ref eval) = evaluator {
                    let state = eval.export_state();
                    println!("{}", serde_json::to_string_pretty(&state).unwrap());
                } else {
                    println!("(no state yet)");
                }
            }
            "" => continue,
            input => {
                accumulated_source.push_str(input);
                accumulated_source.push('\n');

                match parse(&accumulated_source) {
                    Err(e) => eprintln!("Parse error: {e}"),
                    Ok(program) => match analyze(program) {
                        Err(errors) => {
                            for e in &errors {
                                eprintln!("[{}] {}", e.code, e.message);
                            }
                        }
                        Ok(analyzed) => {
                            let mut eval = build_evaluator(&analyzed);
                            for stmt in &analyzed.program.statements {
                                if let Err(e) = eval.exec_statement(stmt) {
                                    eprintln!("Runtime error [{}]: {}", e.code(), e.message());
                                }
                            }
                            evaluator = Some(eval);
                        }
                    }
                }
            }
        }
    }
}
