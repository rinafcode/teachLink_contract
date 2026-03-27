/// Quality metrics collector
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub test_count: usize,
    pub passing_tests: usize,
    pub failing_tests: usize,
    pub code_coverage: f64,
    pub complexity_score: f64,
    pub security_score: f64,
}

pub struct MetricsCollector {
    metrics: HashMap<String, QualityMetrics>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    pub fn record_metrics(&mut self, module: &str, metrics: QualityMetrics) {
        self.metrics.insert(module.to_string(), metrics);
    }

    pub fn get_overall_score(&self) -> f64 {
        if self.metrics.is_empty() {
            return 0.0;
        }

        let total: f64 = self.metrics.values()
            .map(|m| {
                let test_score = if m.test_count > 0 {
                    (m.passing_tests as f64 / m.test_count as f64) * 100.0
                } else {
                    0.0
                };
                (test_score + m.code_coverage + m.security_score) / 3.0
            })
            .sum();

        total / self.metrics.len() as f64
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::from("Quality Metrics Report\n\n");
        
        for (module, metrics) in &self.metrics {
            report.push_str(&format!("Module: {}\n", module));
            report.push_str(&format!("  Tests: {}/{}\n", metrics.passing_tests, metrics.test_count));
            report.push_str(&format!("  Coverage: {:.2}%\n", metrics.code_coverage));
            report.push_str(&format!("  Security: {:.2}%\n\n", metrics.security_score));
        }

        report.push_str(&format!("Overall Score: {:.2}%\n", self.get_overall_score()));
        report
    }
}
