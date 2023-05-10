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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    fn test_on_and_emit() {
        let mut emitter = EventEmitter::new();
        let count = Arc::new(Mutex::new(0));

        let count_cloned = count.clone();
        emitter.on("event", move || {
            *count_cloned.lock().unwrap() += 1;
        });

        emitter.emit("event");
        assert_eq!(*count.lock().unwrap(), 1);

        emitter.emit("event");
        assert_eq!(*count.lock().unwrap(), 2);
    }

    #[test]
    fn test_multiple_listeners() {
        let mut emitter = EventEmitter::new();
        let count1 = Arc::new(Mutex::new(0));
        let count2 = Arc::new(Mutex::new(0));

        let count_cloned = count1.clone();
        emitter.on("event", move || {
            *count_cloned.lock().unwrap() += 1;
        });

        let count_cloned = count2.clone();
        emitter.on("event", move || {
            *count_cloned.lock().unwrap() += 1;
        });

        emitter.emit("event");
        assert_eq!(*count1.lock().unwrap(), 1);
        assert_eq!(*count2.lock().unwrap(), 1);

        emitter.emit("event");
        assert_eq!(*count1.lock().unwrap(), 2);
        assert_eq!(*count2.lock().unwrap(), 2);
    }

    #[test]
    fn test_no_listeners() {
        let emitter = EventEmitter::new();
        emitter.emit("event");
    }

    #[test]
    fn test_different_events() {
        let mut emitter = EventEmitter::new();
        let count1 = Arc::new(Mutex::new(0));
        let count2 = Arc::new(Mutex::new(0));

        let count_cloned = count1.clone();
        emitter.on("event1", move || {
            *count_cloned.lock().unwrap() += 1;
        });

        let count_cloned = count2.clone();
        emitter.on("event2", move || {
            *count_cloned.lock().unwrap() += 1;
        });

        emitter.emit("event1");
        assert_eq!(*count1.lock().unwrap(), 1);

        emitter.emit("event2");
        assert_eq!(*count2.lock().unwrap(), 1);
    }
}
