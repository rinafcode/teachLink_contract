# Incident Response

This document is the operational runbook for responding to production incidents in TeachLink.

## Signals (How You Find Out)

- Prometheus alerts (see `indexer/observability/prometheus/alerts.yml`)
- Grafana dashboards (see `indexer/observability/grafana/dashboards/`)
- Indexer health endpoint: `GET /health`
- Contract telemetry via Soroban events (indexed off-chain)

## Severity

- SEV1: user-impacting outage or data integrity risk (payments/escrow/bridge stuck, indexer down)
- SEV2: degraded performance, partial feature outage, or elevated error rate
- SEV3: minor degradation, noisy alerts, or non-urgent bugs

## First 10 Minutes Checklist

1. Identify which alert fired and the affected component (indexer, DB, Horizon, etc.).
2. Confirm blast radius in Grafana: error rate, latency, event-processing lag.
3. Check indexer `GET /health` for readiness and dependency status.
4. If indexer is down: restart the service and verify alerts resolve.
5. If DB is down: validate DB connectivity and failover/restart per infra playbook.
6. If Horizon is unreachable: validate network/DNS and consider switching endpoints.

## Mitigation Patterns

- Reduce load: disable non-critical cron/jobs, increase backoff, shed traffic at edge.
- Restore service: restart the failing service, roll back last deploy, or deploy a hotfix.
- Protect integrity: pause/disable risky flows if supported (circuit breakers / admin gates).

## Communications

- Announce incident start (SEV + short symptom).
- Provide updates every 15-30 minutes for SEV1/SEV2 until resolved.
- Announce resolution + customer impact summary.

## Post-Incident

- Write a postmortem within 48 hours.
- Add/adjust alerts to catch recurrence earlier.
- Add regression tests for the root cause where applicable.

# TeachLink System Overview

Welcome to the **TeachLink** ecosystem—a decentralized knowledge-sharing platform powered by the Stellar network. This document provides a high-level overview of the entire system architecture, integrating blockchain smart contracts, real-time data indexing, and machine learning-driven recommendations.

---

## 🏗️ The Three Pillars

The TeachLink platform is built on three foundational pillars that work in harmony to deliver a seamless decentralized learning experience.

### 1. Blockchain Core (Soroban Smart Contracts)
The heart of TeachLink resides on the Stellar network. Our smart contracts, written in Rust for the Soroban platform, handle all critical business logic with transparency and security.
- **Key Features:** Cross-chain bridging, BFT consensus, multi-sig escrow, content tokenization (NFTs), and decentralized reputation scoring.
- **Location:** `contracts/`
- **Documentation:** [Architecture Details](docs/ARCHITECTURE.md) | [API Reference](docs/API_REFERENCE.md)

### 2. Data Availability Layer (Indexer Runtime)
To provide a fast and responsive user experience, TeachLink employs a high-performance indexer that monitors the Stellar blockchain in real-time.
- **Tech Stack:** TypeScript, NestJS, PostgreSQL, Prometheus.
- **Key Features:** Event processing, historical data archival, and a robust REST API for client applications.
- **Location:** `indexer/`
- **Documentation:** [Indexer Implementation](indexer/IMPLEMENTATION.md) | [Monitoring Guide](indexer/MONITORING.md)

### 3. Intelligence Layer (Recommendation System)
TeachLink leverages machine learning to personalize the learning journey for every user, ensuring high engagement and effective knowledge transfer.
- **Tech Stack:** Python, TensorFlow/PyTorch, Scikit-learn.
- **Key Features:** Content discovery, personalized learning paths, and predictive reputation modeling.
- **Location:** `recommendation-system/`
- **Documentation:** [ML Architecture](recommendation-system/ARCHITECTURE.md) | [Implementation Guide](recommendation-system/IMPLEMENTATION_GUIDE.md)

---

## 🗺️ Global System Map

The following diagram illustrates how the three pillars interact to form the complete TeachLink ecosystem.

```mermaid
graph TD
    subgraph Users["User Interfaces"]
        UI[Web/Mobile Apps]
    end

    subgraph Intelligence["Intelligence Layer"]
        RS[Recommendation System]
        ML[ML Models]
        RS --- ML
    end

    subgraph Data["Data Layer"]
        IDX[Indexer Runtime]
        DB[(PostgreSQL)]
        PROM[Prometheus/Grafana]
        IDX --- DB
        IDX --- PROM
    end

    subgraph Chain["Blockchain Layer (Stellar)"]
        SC[TeachLink Smart Contracts]
        LEDGER[Stellar Ledger]
        SC --- LEDGER
    end

    UI <-->|"API Requests"| IDX
    UI <-->|"Contract Calls"| SC
    IDX <--"Syncs events"--> SC
    RS <--"Consumes data"--> DB
    UI <--"Personalization"--> RS

    style Chain fill:#e1f5fe,stroke:#01579b
    style Data fill:#f3e5f5,stroke:#4a148c
    style Intelligence fill:#e8f5e9,stroke:#1b5e20
    style Users fill:#fff3e0,stroke:#e65100
```

---

## 🛠️ Technology Stack

| Layer | Technologies |
| :--- | :--- |
| **Smart Contracts** | Rust, Soroban SDK, Stellar CLI |
| **Indexer/Backend** | TypeScript, NestJS, TypeORM, PostgreSQL |
| **Machine Learning** | Python, Pandas, NumPy, Scikit-learn |
| **DevOps** | Docker, GitHub Actions, Prometheus, Grafana |
| **Testing** | Cargo Test, Jest, Property-based Testing |

---

## 🚀 Getting Started

If you are a developer looking to contribute or deploy the system, follow these links:

1.  **Environment Setup:** See the [Onboarding section in README](README.md#onboarding).
2.  **Contract Development:** Explore the [Developer Experience Toolkit](DEVELOPER_EXPERIENCE.md).
3.  **Indexing & Monitoring:** Check the [Observability Guide](OBSERVABILITY.md).
4.  **Contribution:** Read our [Contributing Guidelines](CONTRIBUTING.md).

---

## 📜 Repository Health & Safety

- **Security:** Our security protocols are documented in [SECURITY.md](SECURITY.md).
- **Incident Response:** See [INCIDENT_RESPONSE.md](INCIDENT_RESPONSE.md) for emergency procedures.
- **Testing:** We maintain high standards with our [Testing Platform](TESTING_PLATFORM.md).

---

<div align="center">
  <p>Built with ❤️ by the TeachLink Team</p>
</div>
