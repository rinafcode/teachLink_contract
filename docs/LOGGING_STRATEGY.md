# TeachLink Logging Strategy

## Overview

This document defines the standard logging patterns, formats, and levels across the TeachLink ecosystem. Consistent, structured logging is critical for observability, debugging, and security auditing for both off-chain infrastructure (e.g., the Indexer) and on-chain smart contracts.

## 1. Log Levels

The following standard log levels MUST be used across all off-chain services:

- **`error`**: System failures, critical errors requiring immediate attention, uncaught exceptions, and transaction failures.
- **`warn`**: Non-critical errors, anomalous but recoverable states, deprecated API usage, and retries.
- **`info`**: Normal system operations, business logic milestones (e.g., indexer block processing), startup/shutdown events.
- **`debug`**: Detailed diagnostic information, granular state transitions, payload dumps, and tracing for local development.

*Note: The environment variable `LOG_LEVEL` dictates the minimum severity level output by the application.*

## 2. Structured Logging Format

All off-chain services MUST output logs in **Structured JSON format** when running in production (`NODE_ENV=production`). This ensures seamless ingestion by modern log aggregators and monitoring tools (e.g., Datadog, ELK, CloudWatch).

### Standard JSON Schema

- **`timestamp`**: ISO 8601 UTC string (e.g., `2023-11-20T12:00:00.000Z`)
- **`level`**: The log severity level (e.g., `info`, `error`)
- **`context`**: The module, class, or domain emitting the log (e.g., `EventProcessorService`)
- **`message`**: A concise, human-readable description of the event
- **`data`**: (Optional) Additional contextual structured metadata (e.g., `txHash`, `userId`, `contractId`)

### Example Production Log Entry

```json
{
  "timestamp": "2023-11-20T12:00:05.123Z",
  "level": "info",
  "context": "HorizonService",
  "message": "Successfully processed bridge transaction",
  "data": {
    "txHash": "0xabc123...",
    "ledger": 456789
  }
}
```

In non-production environments, logs may fallback to a human-readable, color-coded console output for better developer experience (DX).

## 3. Smart Contract Logging

Soroban smart contracts operate natively on-chain, where traditional `stdout` logging is not viable for production.

1. **Production Auditing (Events):** Use Soroban `#[contractevent]` structs as the primary mechanism for emitting structured logs. Events inherently provide structured, tamper-evident logging that off-chain indexers consume.
2. **Local Debugging:** Use `env.logs().print(...)` exclusively for local testing and development. These statements should ideally be removed or disabled in production deployments.

## 4. Best Practices

- **Do not log sensitive data:** PII (Personally Identifiable Information), private keys, secrets, and auth tokens MUST NOT be logged.
- **Include correlation IDs:** When processing user requests or multi-step asynchronous tasks, include a correlation identifier in the `data` field to trace the execution path.
- **Keep messages static:** The `message` field should be a static string (e.g., `"User login failed"`), while dynamic variables should be placed in the `data` object to facilitate aggregations and log metrics.