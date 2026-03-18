use crate::adapter::LuminaAdapter;
use crate::value::Value;
use std::sync::{mpsc::{Receiver, Sender}, Mutex};

/// An adapter backed by Rust `mpsc` channels.
/// Receives values from a `Receiver` and optionally sends write-backs
/// through a `Sender`.
pub struct ChannelAdapter {
    entity: String,
    rx: Mutex<Receiver<(String, Value)>>,
    tx: Option<Sender<(String, Value)>>,
}

impl ChannelAdapter {
    pub fn new(
        entity: impl Into<String>,
        rx: Receiver<(String, Value)>,
        tx: Option<Sender<(String, Value)>>,
    ) -> Self {
        Self { entity: entity.into(), rx: Mutex::new(rx), tx }
    }
}

impl LuminaAdapter for ChannelAdapter {
    fn entity_name(&self) -> &str { &self.entity }

    fn poll(&mut self) -> Option<(String, Value)> {
        self.rx.lock().ok()?.try_recv().ok()
    }

    fn on_write(&mut self, f: &str, v: &Value) {
        if let Some(tx) = &self.tx {
            let _ = tx.send((f.to_string(), v.clone()));
        }
    }
}
