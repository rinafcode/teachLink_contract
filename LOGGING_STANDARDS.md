# TeachLink Logging Standards

## Overview
This document outlines the logging standards for the TeachLink project to ensure consistent, structured, and correlatable logs across all services for effective production debugging and monitoring.

## Log Format
All services should output logs in JSON format when running in production environments. In development, pretty-printed logs may be used for readability.

### Common Log Fields
Each log entry should include the following standard fields:

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | string (ISO 8601) | Time when the log was generated |
| `level` | string | Log level (trace, debug, info, warn, error, fatal) |
| `message` | string | Human-readable log message |
| `service` | string | Name of the service generating the log |
| `correlationId` | string | Unique identifier for tracing requests across services |
| `spanId` | string (optional) | Identifier for the specific operation within a trace |
| `traceId` | string (optional) | Identifier for distributed tracing |

## Correlation ID
Correlation IDs are used to trace requests as they flow through multiple services. Each external request should generate or receive a correlation ID that is propagated to all subsequent service calls.

### Generation
- If an incoming request contains a correlation ID (via `X-Correlation-ID` header), it should be used
- Otherwise, a new correlation ID should be generated using UUID v4 or similar unique identifier
- The correlation ID should be attached to all outgoing requests and logged in all service operations

### Propagation
- In HTTP services: Include correlation ID in `X-Correlation-ID` header
- In internal service calls: Pass correlation ID as a parameter or context
- In asynchronous processing: Store correlation ID in async context or message metadata

## Log Levels
Use appropriate log levels to indicate the severity and purpose of log entries:

| Level | When to Use |
|-------|-------------|
| `trace` | Extremely detailed information, typically for development debugging |
| `debug` | Detailed diagnostic information useful for troubleshooting |
| `info` | General operational information about normal activities |
| `warn` | Potentially harmful situations that require attention but don't prevent operation |
| `error` | Error events that prevent normal operation but don't cause service failure |
| `fatal` | Severe errors that cause premature termination of the service |

## Service-Specific Guidelines

### Rust Smart Contracts
- Use `env.log()` for structured logging within Soroban smart contracts
- Log important state transitions, external calls, and error conditions
- Always log error context before panicking using the `log_and_panic!` macro
- Include relevant parameters and state information in logs
- Avoid logging sensitive information (private keys, personal data)

### NestJS Indexer Service
- Use the built-in Pino logger via `@nestjs/pino`
- Include correlation ID in all log entries
- Use child loggers with context for specific operations
- Log entry and exit of significant functions at debug level
- Log external API calls, database operations, and message processing
- Include relevant identifiers (nonce, IDs, addresses) in logs

## Log Aggregation
Logs should be shipped to a centralized log aggregation system (e.g., ELK stack, Splunk, CloudWatch) for:
- Cross-service correlation using correlation IDs
- Real-time alerting on error patterns
- Historical analysis and debugging
- Compliance and audit requirements

### Recommended Log Shipping Configuration
- Use JSON format for easy parsing
- Include service name and version in logs
- Ship logs with minimal delay (near real-time)
- Retain logs according to compliance requirements (minimum 30 days)

## Implementation Examples

### Rust Contract Logging
```rust
// Before panic, log error context
env.log(&format!("Attempting to access non-existent mobile profile for user: {}", user));
panic_with_error!(env, MobilePlatformError::DeviceNotSupported);

// Using the provided macro
log_and_panic!(env, MobilePlatformError::DeviceNotSupported, 
               "Mobile profile not found for user: {}", user);
```

### NestJS Service Logging
```typescript
import { Injectable, Logger } from '@nestjs/common';
import { getCorrelationId } from '../utils/async-storage';

@Injectable()
export class ExampleService {
  private readonly logger = new Logger(ExampleService.name);

  async exampleMethod(id: string): Promise<void> {
    const correlationId = getCorrelationId() || 'unknown';
    
    this.logger.debug('Starting example method', { 
      correlationId,
      method: 'exampleMethod',
      inputId: id 
    });

    try {
      // ... implementation ...
      
      this.logger.debug('Completed example method successfully', { 
        correlationId,
        method: 'exampleMethod',
        inputId: id 
      });
    } catch (error) {
      this.logger.error('Failed in example method', { 
        correlationId,
        method: 'exampleMethod',
        inputId: id,
        error: error.message,
        stack: error.stack 
      });
      throw error;
    }
  }
}
```

## Configuration
Log behavior can be configured via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `LOG_LEVEL` | Minimum log level to output | `info` |
| `LOG_FORMAT` | Output format: `json` or `pretty` | `json` |
| `NODE_ENV` | Environment: `development` or `production` | `development` |

In development, setting `NODE_ENV=development` will enable pretty-printed logs for easier local debugging.

## Security Considerations
- Never log sensitive information such as:
  - Private keys or secrets
  - Personal identifiable information (PII) beyond what's necessary
  - Authentication tokens or credentials
- Review logs regularly to ensure no sensitive data is being exposed
- Use log masking or redaction for sensitive fields when necessary

## Monitoring and Alerting
Set up alerts based on log patterns:
- Error rate thresholds
- Specific error messages indicating critical issues
- Performance anomalies detected through log analysis
- Failed external service dependencies

## References
- Soroban SDK Documentation: https://soroban.stellar.org/api/
- NestJS Pino Documentation: https://github.com/nestjs/pino
- Distributed Tracing Concepts: https://opentelemetry.io/docs/concepts/