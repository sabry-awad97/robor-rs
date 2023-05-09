use std::collections::HashMap;

pub struct EventEmitter {
    listeners: HashMap<String, Vec<Box<dyn Fn()>>>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
        }
    }

    pub fn on<F>(&mut self, event_name: &str, listener: F)
    where
        F: Fn() + 'static + Send + Sync,
    {
        self.listeners
            .entry(event_name.to_string())
            .or_insert(Vec::new())
            .push(Box::new(listener));
    }

    pub fn emit(&self, event_name: &str) {
        if let Some(listeners) = self.listeners.get(event_name) {
            for listener in listeners {
                listener();
            }
        }
    }
}
