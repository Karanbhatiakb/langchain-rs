use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::traits::CallbackHandler;

#[derive(Debug, Clone)]
pub struct EvaluationCallbackHandler {
    metrics: Arc<Mutex<HashMap<String, Vec<f64>>>>,
}

impl EvaluationCallbackHandler {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_metric(&self, name: &str, value: f64) {
        self.metrics
            .lock()
            .unwrap()
            .entry(name.to_string())
            .or_default()
            .push(value);
    }

    pub fn get_metrics(&self) -> HashMap<String, Vec<f64>> {
        self.metrics.lock().unwrap().clone()
    }

    pub fn get_average(&self, name: &str) -> Option<f64> {
        let metrics = self.metrics.lock().unwrap();
        metrics.get(name).and_then(|values| {
            if values.is_empty() {
                None
            } else {
                Some(values.iter().sum::<f64>() / values.len() as f64)
            }
        })
    }
}

impl Default for EvaluationCallbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CallbackHandler for EvaluationCallbackHandler {
    fn on_chain_end(&self, _name: &str, outputs: &Value) {
        if let Some(obj) = outputs.as_object() {
            for (key, val) in obj {
                if let Some(n) = val.as_f64() {
                    self.add_metric(key, n);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_handler_new() {
        let h = EvaluationCallbackHandler::new();
        assert!(h.get_metrics().is_empty());
    }

    #[test]
    fn test_eval_handler_default() {
        let h = EvaluationCallbackHandler::default();
        assert!(h.get_metrics().is_empty());
    }

    #[test]
    fn test_eval_handler_add_metric() {
        let h = EvaluationCallbackHandler::new();
        h.add_metric("accuracy", 0.95);
        h.add_metric("accuracy", 0.97);
        let metrics = h.get_metrics();
        assert_eq!(metrics.get("accuracy").unwrap(), &vec![0.95, 0.97]);
    }

    #[test]
    fn test_eval_handler_get_average() {
        let h = EvaluationCallbackHandler::new();
        h.add_metric("score", 1.0);
        h.add_metric("score", 2.0);
        h.add_metric("score", 3.0);
        let avg = h.get_average("score");
        assert!((avg.unwrap() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_eval_handler_get_average_missing() {
        let h = EvaluationCallbackHandler::new();
        assert!(h.get_average("nonexistent").is_none());
    }

    #[test]
    fn test_eval_handler_on_chain_end() {
        let h = EvaluationCallbackHandler::new();
        let mut map = serde_json::Map::new();
        map.insert("accuracy".into(), Value::Number(serde_json::Number::from_f64(0.9).unwrap()));
        map.insert("loss".into(), Value::Number(serde_json::Number::from_f64(0.1).unwrap()));
        let outputs = Value::Object(map);
        h.on_chain_end("eval", &outputs);
        let metrics = h.get_metrics();
        assert_eq!(metrics.get("accuracy").unwrap(), &vec![0.9]);
        assert_eq!(metrics.get("loss").unwrap(), &vec![0.1]);
    }

    #[test]
    fn test_eval_handler_on_chain_end_skips_non_float() {
        let h = EvaluationCallbackHandler::new();
        let mut map = serde_json::Map::new();
        map.insert("name".into(), Value::String("test".into()));
        let outputs = Value::Object(map);
        h.on_chain_end("eval", &outputs);
        assert!(h.get_metrics().is_empty());
    }

    #[test]
    fn test_eval_handler_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<EvaluationCallbackHandler>();
        assert_sync::<EvaluationCallbackHandler>();
    }
}
