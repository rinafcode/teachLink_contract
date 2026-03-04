/// Performance benchmark runner
use std::time::{Duration, Instant};
use std::collections::HashMap;

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

pub struct BenchmarkRunner {
    results: HashMap<String, BenchmarkResult>,
    warmup_iterations: u64,
    test_iterations: u64,
}

impl BenchmarkRunner {
    pub fn new(warmup_iterations: u64, test_iterations: u64) -> Self {
        Self {
            results: HashMap::new(),
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

    pub fn compare_with_baseline(&self, baseline: &BenchmarkRunner) {
        println!("\n=== Comparison with Baseline ===\n");
        
        for (name, current) in &self.results {
            if let Some(baseline_result) = baseline.get_result(name) {
                let diff_pct = ((current.avg_duration.as_nanos() as f64 
                    - baseline_result.avg_duration.as_nanos() as f64) 
                    / baseline_result.avg_duration.as_nanos() as f64) * 100.0;
                
                let status = if diff_pct > 5.0 {
                    "⚠️  SLOWER"
                } else if diff_pct < -5.0 {
                    "✅ FASTER"
                } else {
                    "➡️  SIMILAR"
                };
                
                println!("{} {}: {:.2}%", status, name, diff_pct);
            }
        }
    }

    pub fn export_json(&self) -> String {
        let mut json = String::from("{\n");
        json.push_str("  \"benchmarks\": [\n");
        
        for (i, (name, result)) in self.results.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!("      \"name\": \"{}\",\n", name));
            json.push_str(&format!("      \"iterations\": {},\n", result.iterations));
            json.push_str(&format!("      \"avg_ns\": {},\n", result.avg_duration.as_nanos()));
            json.push_str(&format!("      \"min_ns\": {},\n", result.min_duration.as_nanos()));
            json.push_str(&format!("      \"max_ns\": {},\n", result.max_duration.as_nanos()));
            json.push_str(&format!("      \"p50_ns\": {},\n", result.p50.as_nanos()));
            json.push_str(&format!("      \"p95_ns\": {},\n", result.p95.as_nanos()));
            json.push_str(&format!("      \"p99_ns\": {}\n", result.p99.as_nanos()));
            json.push_str("    }");
            
            if i < self.results.len() - 1 {
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
}
