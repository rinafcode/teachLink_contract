use std::collections::HashMap;
/// Performance benchmark runner with gas tracking support
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: u64,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

#[derive(Debug, Clone)]
pub struct GasBenchmarkResult {
    pub name: String,
    pub gas_used: u64,
    pub threshold: u64,
    pub within_threshold: bool,
}

pub struct BenchmarkRunner {
    results: HashMap<String, BenchmarkResult>,
    gas_results: HashMap<String, GasBenchmarkResult>,
    warmup_iterations: u64,
    test_iterations: u64,
}

impl BenchmarkRunner {
    pub fn new(warmup_iterations: u64, test_iterations: u64) -> Self {
        Self {
            results: HashMap::new(),
            gas_results: HashMap::new(),
            warmup_iterations,
            test_iterations,
        }
    }

    pub fn benchmark<F>(&mut self, name: &str, mut f: F)
    where
        F: FnMut(),
    {
        // Warmup
        for _ in 0..self.warmup_iterations {
            f();
        }

        // Actual benchmark
        let mut durations = Vec::with_capacity(self.test_iterations as usize);

        for _ in 0..self.test_iterations {
            let start = Instant::now();
            f();
            let duration = start.elapsed();
            durations.push(duration);
        }

        // Calculate statistics
        durations.sort();
        let total: Duration = durations.iter().sum();
        let avg = total / self.test_iterations as u32;
        let min = *durations.first().unwrap();
        let max = *durations.last().unwrap();

        let p50_idx = (self.test_iterations as f64 * 0.50) as usize;
        let p95_idx = (self.test_iterations as f64 * 0.95) as usize;
        let p99_idx = (self.test_iterations as f64 * 0.99) as usize;

        let result = BenchmarkResult {
            name: name.to_string(),
            iterations: self.test_iterations,
            total_duration: total,
            avg_duration: avg,
            min_duration: min,
            max_duration: max,
            p50: durations[p50_idx],
            p95: durations[p95_idx],
            p99: durations[p99_idx],
        };

        self.results.insert(name.to_string(), result);
    }

    /// Record a gas measurement for a contract operation.
    pub fn record_gas(&mut self, name: &str, gas_used: u64, threshold: u64) {
        let result = GasBenchmarkResult {
            name: name.to_string(),
            gas_used,
            threshold,
            within_threshold: gas_used <= threshold,
        };
        self.gas_results.insert(name.to_string(), result);
    }

    /// Get a gas benchmark result by name.
    pub fn get_gas_result(&self, name: &str) -> Option<&GasBenchmarkResult> {
        self.gas_results.get(name)
    }

    /// Check if all gas benchmarks are within thresholds.
    pub fn all_gas_within_thresholds(&self) -> bool {
        self.gas_results.values().all(|r| r.within_threshold)
    }

    /// Get all gas regressions (operations exceeding thresholds).
    pub fn get_gas_regressions(&self) -> Vec<&GasBenchmarkResult> {
        self.gas_results
            .values()
            .filter(|r| !r.within_threshold)
            .collect()
    }

    pub fn get_result(&self, name: &str) -> Option<&BenchmarkResult> {
        self.results.get(name)
    }

    pub fn print_results(&self) {
        println!("\n=== Benchmark Results ===\n");

        for (name, result) in &self.results {
            println!("Benchmark: {}", name);
            println!("  Iterations: {}", result.iterations);
            println!("  Average:    {:?}", result.avg_duration);
            println!("  Min:        {:?}", result.min_duration);
            println!("  Max:        {:?}", result.max_duration);
            println!("  P50:        {:?}", result.p50);
            println!("  P95:        {:?}", result.p95);
            println!("  P99:        {:?}", result.p99);
            println!();
        }
    }

    pub fn print_gas_results(&self) {
        println!("\n=== Gas Benchmark Results ===\n");
        println!(
            "  {:<35} | {:>12} | {:>12} | {}",
            "Operation", "Gas Used", "Threshold", "Status"
        );
        println!("  {:->35}-+-{:->12}-+-{:->12}-+{:->8}", "", "", "", "");

        for (name, result) in &self.gas_results {
            let status = if result.within_threshold {
                "PASS"
            } else {
                "FAIL"
            };
            println!(
                "  {:<35} | {:>12} | {:>12} | {}",
                name, result.gas_used, result.threshold, status
            );
        }
        println!();
    }

    pub fn compare_with_baseline(&self, baseline: &BenchmarkRunner) {
        println!("\n=== Comparison with Baseline ===\n");

        for (name, current) in &self.results {
            if let Some(baseline_result) = baseline.get_result(name) {
                let diff_pct = ((current.avg_duration.as_nanos() as f64
                    - baseline_result.avg_duration.as_nanos() as f64)
                    / baseline_result.avg_duration.as_nanos() as f64)
                    * 100.0;

                let status = if diff_pct > 5.0 {
                    "SLOWER"
                } else if diff_pct < -5.0 {
                    "FASTER"
                } else {
                    "SIMILAR"
                };

                println!("{} {}: {:.2}%", status, name, diff_pct);
            }
        }
    }

    /// Compare gas results with a baseline runner.
    pub fn compare_gas_with_baseline(&self, baseline: &BenchmarkRunner) {
        println!("\n=== Gas Comparison with Baseline ===\n");

        for (name, current) in &self.gas_results {
            if let Some(baseline_result) = baseline.get_gas_result(name) {
                if baseline_result.gas_used > 0 {
                    let diff_pct = ((current.gas_used as f64 - baseline_result.gas_used as f64)
                        / baseline_result.gas_used as f64)
                        * 100.0;

                    let status = if diff_pct > 10.0 {
                        "REGRESSION"
                    } else if diff_pct > 5.0 {
                        "WARNING"
                    } else if diff_pct < -5.0 {
                        "IMPROVED"
                    } else {
                        "STABLE"
                    };

                    println!(
                        "  {} {}: {} -> {} ({:+.1}%)",
                        status, name, baseline_result.gas_used, current.gas_used, diff_pct
                    );
                }
            }
        }
    }

    pub fn export_json(&self) -> String {
        let mut json = String::from("{\n");
        json.push_str("  \"timing_benchmarks\": [\n");

        for (i, (name, result)) in self.results.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!("      \"name\": \"{}\",\n", name));
            json.push_str(&format!("      \"iterations\": {},\n", result.iterations));
            json.push_str(&format!(
                "      \"avg_ns\": {},\n",
                result.avg_duration.as_nanos()
            ));
            json.push_str(&format!(
                "      \"min_ns\": {},\n",
                result.min_duration.as_nanos()
            ));
            json.push_str(&format!(
                "      \"max_ns\": {},\n",
                result.max_duration.as_nanos()
            ));
            json.push_str(&format!("      \"p50_ns\": {},\n", result.p50.as_nanos()));
            json.push_str(&format!("      \"p95_ns\": {},\n", result.p95.as_nanos()));
            json.push_str(&format!("      \"p99_ns\": {}\n", result.p99.as_nanos()));
            json.push_str("    }");

            if i < self.results.len() - 1 {
                json.push_str(",");
            }
            json.push_str("\n");
        }

        json.push_str("  ],\n");
        json.push_str("  \"gas_benchmarks\": [\n");

        let gas_items: Vec<_> = self.gas_results.iter().collect();
        for (i, (name, result)) in gas_items.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!("      \"name\": \"{}\",\n", name));
            json.push_str(&format!("      \"gas_used\": {},\n", result.gas_used));
            json.push_str(&format!("      \"threshold\": {},\n", result.threshold));
            json.push_str(&format!(
                "      \"within_threshold\": {}\n",
                result.within_threshold
            ));
            json.push_str("    }");

            if i < gas_items.len() - 1 {
                json.push_str(",");
            }
            json.push_str("\n");
        }

        json.push_str("  ]\n");
        json.push_str("}\n");
        json
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_benchmark_runner() {
        let mut runner = BenchmarkRunner::new(10, 100);

        runner.benchmark("sleep_1ms", || {
            thread::sleep(Duration::from_micros(100));
        });

        let result = runner.get_result("sleep_1ms").unwrap();
        assert_eq!(result.iterations, 100);
        assert!(result.avg_duration.as_micros() >= 100);
    }

    #[test]
    fn test_multiple_benchmarks() {
        let mut runner = BenchmarkRunner::new(5, 50);

        runner.benchmark("fast", || {
            let _ = 1 + 1;
        });

        runner.benchmark("slow", || {
            thread::sleep(Duration::from_micros(10));
        });

        assert!(runner.results.len() == 2);
    }

    #[test]
    fn test_gas_tracking() {
        let mut runner = BenchmarkRunner::new(0, 0);

        runner.record_gas("initialize", 350_000, 500_000);
        runner.record_gas("bridge_out", 900_000, 800_000);
        runner.record_gas("read_query", 50_000, 100_000);

        assert!(
            runner
                .get_gas_result("initialize")
                .unwrap()
                .within_threshold
        );
        assert!(
            !runner
                .get_gas_result("bridge_out")
                .unwrap()
                .within_threshold
        );
        assert!(
            runner
                .get_gas_result("read_query")
                .unwrap()
                .within_threshold
        );

        assert!(!runner.all_gas_within_thresholds());
        assert_eq!(runner.get_gas_regressions().len(), 1);
    }

    #[test]
    fn test_gas_comparison() {
        let mut baseline = BenchmarkRunner::new(0, 0);
        baseline.record_gas("op1", 1000, 2000);
        baseline.record_gas("op2", 500, 1000);

        let mut current = BenchmarkRunner::new(0, 0);
        current.record_gas("op1", 1100, 2000); // 10% increase
        current.record_gas("op2", 450, 1000); // 10% decrease

        // op1 should show as warning (10% > 5%)
        // op2 should show as improved (-10% < -5%)
        // Just verify they exist
        assert!(current.get_gas_result("op1").is_some());
        assert!(current.get_gas_result("op2").is_some());
    }
}
