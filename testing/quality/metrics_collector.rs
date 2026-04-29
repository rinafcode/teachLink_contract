/// Quality metrics collector
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QualityMetrics {
    pub test_count: usize,
    pub passing_tests: usize,
    pub failing_tests: usize,
    pub code_coverage: f64,
    pub complexity_score: f64,
    pub cognitive_complexity: f64,
    pub duplication_percentage: f64,
    pub security_score: f64,
    pub loc: usize,
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

    pub fn load_from_reports(&mut self, reports_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Logic to parse reports/coverage.json, reports/duplication.json, etc.
        // For now, we simulate the aggregation
        Ok(())
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
                
                let duplication_penalty = m.duplication_percentage * 2.0;
                let complexity_penalty = (m.complexity_score - 10.0).max(0.0) * 0.5;
                
                let base_score = (test_score + m.code_coverage + m.security_score) / 3.0;
                (base_score - duplication_penalty - complexity_penalty).max(0.0)
            })
            .sum();

        total / self.metrics.len() as f64
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::from("# Comprehensive Quality Metrics Report\n\n");
        report.push_str(&format!("Generated at: {}\n\n", chrono::Utc::now().to_rfc3339()));
        
        report.push_str("| Module | Tests | Coverage | Complexity | Duplication | Score |\n");
        report.push_str("|--------|-------|----------|------------|-------------|-------|\n");

        for (module, metrics) in &self.metrics {
            let score = (metrics.passing_tests as f64 / metrics.test_count.max(1) as f64 * 40.0) +
                        (metrics.code_coverage * 0.4) +
                        (metrics.security_score * 0.2);

            report.push_str(&format!("| {} | {}/{} | {:.2}% | {:.2} | {:.2}% | {:.2}% |\n",
                module, metrics.passing_tests, metrics.test_count,
                metrics.code_coverage, metrics.complexity_score,
                metrics.duplication_percentage, score));
        }

        report.push_str(&format!("\n**Overall Quality Score: {:.2}%**\n", self.get_overall_score()));
        report
    }
}
