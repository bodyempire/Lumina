#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Instant;

#[cfg(target_arch = "wasm32")]
impl Instant {
    pub fn now() -> Self { Instant }
    pub fn elapsed(&self) -> std::time::Duration { std::time::Duration::from_secs(0) }
}
use std::collections::HashMap;
use lumina_parser::ast::{RuleDecl, RuleTrigger};

/// A pending `for` timer — fires when condition holds for the full duration
#[derive(Debug)]
pub struct ForTimer {
    pub rule_name:     String,
    pub instance_name: String,
    pub started_at:    Instant,
    pub duration_secs: f64,
}

impl ForTimer {
    pub fn has_elapsed(&self) -> bool {
        self.started_at.elapsed().as_secs_f64() >= self.duration_secs
    }
}

/// A pending `every` timer — fires repeatedly on a fixed interval
#[derive(Debug)]
pub struct EveryTimer {
    pub rule_name:    String,
    pub interval_secs: f64,
    pub last_fired:   Instant,
}

impl EveryTimer {
    pub fn is_due(&self) -> bool {
        self.last_fired.elapsed().as_secs_f64() >= self.interval_secs
    }

    pub fn reset(&mut self) {
        self.last_fired = Instant::now();
    }
}

/// Manages all active timers in the runtime
pub struct TimerHeap {
    pub for_timers:   HashMap<String, ForTimer>,
    pub every_timers: HashMap<String, EveryTimer>,
}

impl TimerHeap {
    pub fn new() -> Self {
        Self {
            for_timers:   HashMap::new(),
            every_timers: HashMap::new(),
        }
    }

    /// Register all `every` rules at startup — they begin ticking immediately
    pub fn register_every_rules(&mut self, rules: &[RuleDecl]) {
        for rule in rules {
            if let RuleTrigger::Every(duration) = &rule.trigger {
                self.every_timers.insert(
                    rule.name.clone(),
                    EveryTimer {
                        rule_name:     rule.name.clone(),
                        interval_secs: duration.to_seconds(),
                        last_fired:    Instant::now(),
                    },
                );
            }
        }
    }

    /// Start a `for` timer for a rule+instance pair.
    /// If a timer already exists for this pair, do nothing (don't reset).
    pub fn start_for_timer(
        &mut self,
        rule_name: &str,
        instance_name: &str,
        duration_secs: f64,
    ) -> Result<(), String> {
        let key = format!("{rule_name}::{instance_name}");
        if !self.for_timers.contains_key(&key) {
            self.for_timers.insert(key, ForTimer {
                rule_name:     rule_name.to_string(),
                instance_name: instance_name.to_string(),
                started_at:    Instant::now(),
                duration_secs,
            });
        }
        Ok(())
    }

    /// Cancel a `for` timer — called when the condition becomes false
    pub fn cancel_for_timer(&mut self, rule_name: &str, instance_name: &str) {
        let key = format!("{rule_name}::{instance_name}");
        self.for_timers.remove(&key);
    }

    /// Collect all elapsed `for` timers — returns them and removes from heap
    pub fn drain_elapsed_for_timers(&mut self) -> Vec<ForTimer> {
        let elapsed_keys: Vec<String> = self.for_timers
            .iter()
            .filter(|(_, t)| t.has_elapsed())
            .map(|(k, _)| k.clone())
            .collect();
        elapsed_keys.into_iter()
            .filter_map(|k| self.for_timers.remove(&k))
            .collect()
    }

    /// Collect all due `every` timers — resets them and returns rule names
    pub fn drain_due_every_timers(&mut self) -> Vec<String> {
        let due: Vec<String> = self.every_timers
            .iter()
            .filter(|(_, t)| t.is_due())
            .map(|(k, _)| k.clone())
            .collect();
        for key in &due {
            if let Some(timer) = self.every_timers.get_mut(key) {
                timer.reset();
            }
        }
        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use lumina_parser::ast::*;
    use lumina_lexer::token::Span;

    #[test]
    fn test_for_timer_elapsed() {
        let timer = ForTimer {
            rule_name: "test".to_string(),
            instance_name: "inst".to_string(),
            started_at: Instant::now(),
            duration_secs: 0.05,
        };
        assert!(!timer.has_elapsed());
        thread::sleep(std::time::Duration::from_millis(60));
        assert!(timer.has_elapsed());
    }

    #[test]
    fn test_every_timer_due() {
        let mut timer = EveryTimer {
            rule_name: "check".to_string(),
            interval_secs: 0.05,
            last_fired: Instant::now(),
        };
        assert!(!timer.is_due());
        thread::sleep(std::time::Duration::from_millis(60));
        assert!(timer.is_due());
        timer.reset();
        assert!(!timer.is_due());
    }

    #[test]
    fn test_for_timer_no_reset() {
        let mut heap = TimerHeap::new();
        heap.start_for_timer("r1", "i1", 1.0).unwrap();
        let t1 = heap.for_timers.get("r1::i1").unwrap().started_at;

        thread::sleep(std::time::Duration::from_millis(10));
        heap.start_for_timer("r1", "i1", 1.0).unwrap();
        let t2 = heap.for_timers.get("r1::i1").unwrap().started_at;

        assert_eq!(t1, t2, "timer should not be reset");
    }

    #[test]
    fn test_cancel_for_timer() {
        let mut heap = TimerHeap::new();
        heap.start_for_timer("r1", "i1", 1.0).unwrap();
        assert!(heap.for_timers.contains_key("r1::i1"));

        heap.cancel_for_timer("r1", "i1");
        assert!(!heap.for_timers.contains_key("r1::i1"));
    }

    #[test]
    fn test_drain_elapsed() {
        let mut heap = TimerHeap::new();
        heap.start_for_timer("fast", "i1", 0.05).unwrap();
        heap.start_for_timer("slow", "i1", 10.0).unwrap();

        thread::sleep(std::time::Duration::from_millis(60));
        let elapsed = heap.drain_elapsed_for_timers();

        assert_eq!(elapsed.len(), 1);
        assert_eq!(elapsed[0].rule_name, "fast");
        assert!(heap.for_timers.contains_key("slow::i1"));
    }

    #[test]
    fn test_register_every_rules() {
        let rules = vec![
            RuleDecl {
                name: "hourly".to_string(),
                trigger: RuleTrigger::Every(Duration {
                    value: 1.0,
                    unit: TimeUnit::Hours,
                }),
                actions: vec![],
                cooldown: None,
                on_clear: None,
                span: Span::default(),
            },
            RuleDecl {
                name: "on_change".to_string(),
                trigger: RuleTrigger::When(vec![Condition {
                    expr: Expr::Bool(true),
                    becomes: None,
                    for_duration: None,
                    frequency: None,
                }]),
                actions: vec![],
                cooldown: None,
                on_clear: None,
                span: Span::default(),
            },
        ];

        let mut heap = TimerHeap::new();
        heap.register_every_rules(&rules);

        assert_eq!(heap.every_timers.len(), 1);
        assert!(heap.every_timers.contains_key("hourly"));
        assert_eq!(heap.every_timers["hourly"].interval_secs, 3600.0);
    }
}
