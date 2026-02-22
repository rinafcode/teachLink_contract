/// Code coverage analyzer
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub total_lines: usize,
    pub covered_lines: usize,
    pub total_functions: usize,
    pub covered_functions: usize,
    pub coverage_percentage: f64,
    pub file_coverage: HashMap<String, FileCoverage>,
}

#[derive(Debug, Clone)]
pub struct FileCoverage {
    pub path: String,
    pub total_lines: usize,
    pub covered_lines: usize,
    pub uncovered_lines: Vec<usize>,
    pub coverage_percentage: f64,
}

pub struct CoverageAnalyzer {
    covered_lines: HashMap<String, HashSet<usize>>,
    total_lines: HashMap<String, usize>,
}

impl CoverageAnalyzer {
    pub fn new() -> Self {
        Self {
            covered_lines: HashMap::new(),
            total_lines: HashMap::new(),
        }
    }

    pub fn mark_line_covered(&mut self, file: &str, line: usize) {
        self.covered_lines
            .entry(file.to_string())
            .or_insert_with(HashSet::new)
            .insert(line);
    }

    pub fn analyze_file(&mut self, file_path: &Path) -> Result<(), String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let executable_lines = self.count_executable_lines(&content);
        self.total_lines.insert(
            file_path.to_string_lossy().to_string(),
            executable_lines,
        );

        Ok(())
    }

    fn count_executable_lines(&self, content: &str) -> usize {
        content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty()
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with("/*")
                    && !trimmed.starts_with('*')
                    && !trimmed.starts_with('}')
                    && !trimmed.starts_with('{')
            })
            .count()
    }

    pub fn generate_report(&self) -> CoverageReport {
        let mut file_coverage = HashMap::new();
        let mut total_lines = 0;
        let mut covered_lines = 0;

        for (file, &total) in &self.total_lines {
            let covered = self.covered_lines
                .get(file)
                .map(|set| set.len())
                .unwrap_or(0);

            let uncovered: Vec<usize> = (1..=total)
                .filter(|line| {
                    !self.covered_lines
                        .get(file)
                        .map(|set| set.contains(line))
                        .unwrap_or(false)
                })
                .collect();

            let coverage_pct = if total > 0 {
                (covered as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            file_coverage.insert(
                file.clone(),
                FileCoverage {
                    path: file.clone(),
                    total_lines: total,
                    covered_lines: covered,
                    uncovered_lines: uncovered,
                    coverage_percentage: coverage_pct,
                },
            );

            total_lines += total;
            covered_lines += covered;
        }

        let coverage_percentage = if total_lines > 0 {
            (covered_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        CoverageReport {
            total_lines,
            covered_lines,
            total_functions: 0,
            covered_functions: 0,
            coverage_percentage,
            file_coverage,
        }
    }
}
