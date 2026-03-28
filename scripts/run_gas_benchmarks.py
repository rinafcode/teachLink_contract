#!/usr/bin/env python3
"""
Gas benchmark runner for TeachLink contract.

Runs gas benchmarks, compares results against a baseline,
and generates regression reports.

Usage:
    python3 scripts/run_gas_benchmarks.py [--update-baseline] [--output report.json]
"""

import json
import os
import subprocess
import sys
import re
from datetime import datetime
from pathlib import Path

REPO_ROOT = Path(__file__).parent.parent
THRESHOLDS_FILE = REPO_ROOT / "gas_thresholds.json"
BASELINE_FILE = REPO_ROOT / "gas_baseline.json"
DEFAULT_OUTPUT = REPO_ROOT / "gas_benchmark_report.json"


def load_json(path: Path) -> dict:
    """Load a JSON file, returning empty dict if not found."""
    if path.exists():
        with open(path) as f:
            return json.load(f)
    return {}


def save_json(path: Path, data: dict):
    """Save data to a JSON file."""
    with open(path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"  Saved: {path}")


def run_cargo_bench() -> str:
    """Run cargo test with gas benchmarks and capture output."""
    print("  Running gas benchmarks...")
    try:
        result = subprocess.run(
            ["cargo", "test", "--release", "-p", "teachlink-contract",
             "--test", "test_gas_benchmarks", "--", "--nocapture"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            timeout=600,
        )
        return result.stdout + "\n" + result.stderr
    except subprocess.TimeoutExpired:
        print("  ERROR: Benchmark timed out after 600 seconds")
        return ""
    except FileNotFoundError:
        print("  ERROR: cargo not found. Ensure Rust toolchain is installed.")
        return ""


def parse_gas_output(output: str) -> dict:
    """Parse gas measurement output from test runs."""
    results = {}
    # Match lines like: [GAS] operation_name: 12345 instructions (threshold: 500000)
    pattern = r"\[GAS\]\s+(\S+):\s+(\d+)\s+instructions\s+\(threshold:\s+(\d+)\)"
    for match in re.finditer(pattern, output):
        name = match.group(1)
        gas_used = int(match.group(2))
        threshold = int(match.group(3))
        results[name] = {
            "gas_used": gas_used,
            "threshold": threshold,
            "within_threshold": gas_used <= threshold,
        }
    return results


def check_wasm_size(thresholds: dict) -> dict:
    """Check WASM binary size against threshold."""
    wasm_path = REPO_ROOT / "target" / "wasm32-unknown-unknown" / "release" / "teachlink_contract.wasm"
    result = {"exists": False}

    if wasm_path.exists():
        size = wasm_path.stat().st_size
        wasm_thresholds = thresholds.get("thresholds", {}).get("wasm_binary", {})
        max_size = wasm_thresholds.get("max_size_bytes", 307200)
        warning_size = wasm_thresholds.get("warning_size_bytes", 256000)

        result = {
            "exists": True,
            "size_bytes": size,
            "max_bytes": max_size,
            "warning_bytes": warning_size,
            "within_threshold": size <= max_size,
            "needs_warning": size > warning_size,
        }

        if size > max_size:
            print(f"  WASM SIZE REGRESSION: {size} bytes exceeds {max_size} bytes")
        elif size > warning_size:
            print(f"  WASM SIZE WARNING: {size} bytes approaching {max_size} bytes limit")
        else:
            print(f"  WASM size OK: {size} bytes (limit: {max_size})")

    return result


def compare_with_baseline(current: dict, baseline: dict) -> list:
    """Compare current results with baseline, returning regressions."""
    regressions = []
    for name, data in current.items():
        if name in baseline:
            base_gas = baseline[name].get("gas_used", 0)
            curr_gas = data["gas_used"]
            if base_gas > 0:
                pct_change = ((curr_gas - base_gas) / base_gas) * 100
                data["baseline_gas"] = base_gas
                data["pct_change"] = round(pct_change, 2)

                if pct_change > 10:
                    regressions.append({
                        "operation": name,
                        "baseline": base_gas,
                        "current": curr_gas,
                        "change_pct": round(pct_change, 2),
                        "severity": "critical" if pct_change > 25 else "warning",
                    })
    return regressions


def generate_report(results: dict, wasm_info: dict, regressions: list) -> dict:
    """Generate the benchmark report."""
    return {
        "generated_at": datetime.utcnow().isoformat() + "Z",
        "summary": {
            "total_operations": len(results),
            "passed": sum(1 for r in results.values() if r.get("within_threshold", True)),
            "failed": sum(1 for r in results.values() if not r.get("within_threshold", True)),
            "regressions": len(regressions),
        },
        "operations": results,
        "wasm_binary": wasm_info,
        "regressions": regressions,
    }


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Gas benchmark runner")
    parser.add_argument("--update-baseline", action="store_true",
                        help="Update baseline with current results")
    parser.add_argument("--output", type=str, default=str(DEFAULT_OUTPUT),
                        help="Output report path")
    args = parser.parse_args()

    print("=" * 50)
    print("  TeachLink Gas Benchmark Runner")
    print("=" * 50)

    # Load thresholds
    thresholds = load_json(THRESHOLDS_FILE)
    if not thresholds:
        print("  WARNING: No gas_thresholds.json found, using defaults")

    # Run benchmarks
    output = run_cargo_bench()
    results = parse_gas_output(output)

    if not results:
        print("  No gas measurements captured. Check test output.")
        sys.exit(1)

    # Check WASM size
    wasm_info = check_wasm_size(thresholds)

    # Compare with baseline
    baseline = load_json(BASELINE_FILE)
    regressions = compare_with_baseline(results, baseline)

    # Generate report
    report = generate_report(results, wasm_info, regressions)
    save_json(Path(args.output), report)

    # Print summary
    print("\n" + "=" * 50)
    print("  BENCHMARK SUMMARY")
    print("=" * 50)
    print(f"  Operations tested:  {report['summary']['total_operations']}")
    print(f"  Passed:             {report['summary']['passed']}")
    print(f"  Failed:             {report['summary']['failed']}")
    print(f"  Regressions:        {report['summary']['regressions']}")

    if regressions:
        print("\n  REGRESSIONS DETECTED:")
        for r in regressions:
            severity = "CRITICAL" if r["severity"] == "critical" else "WARNING"
            print(f"    [{severity}] {r['operation']}: "
                  f"{r['baseline']} -> {r['current']} ({r['change_pct']:+.1f}%)")

    # Update baseline if requested
    if args.update_baseline:
        baseline_data = {k: {"gas_used": v["gas_used"], "threshold": v["threshold"]}
                         for k, v in results.items()}
        baseline_data["updated_at"] = datetime.utcnow().isoformat() + "Z"
        save_json(BASELINE_FILE, baseline_data)
        print("\n  Baseline updated.")

    # Exit code
    has_failures = report["summary"]["failed"] > 0 or any(
        r["severity"] == "critical" for r in regressions
    )
    if has_failures:
        print("\n  RESULT: FAILED - Gas regressions detected")
        sys.exit(1)
    else:
        print("\n  RESULT: PASSED")
        sys.exit(0)


if __name__ == "__main__":
    main()
