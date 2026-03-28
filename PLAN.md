# Issue #174: Insufficient Logging - Implementation Plan

## Problem
Insufficient logging for production debugging across services.

## Solution Overview
Implement structured logging with correlation IDs for all services:
1. Rust contracts: Add Env.log() calls before panic_with_error! and create logging macros
2. NestJS indexer: Enhance with nestjs-pino, correlation ID middleware, and structured logging
3. Configure log aggregation and monitoring
4. Document logging standards

## Implementation Details

### 1. Rust Contracts Logging Enhancement

#### Create Logging Macro
Add a logging macro in `contracts/teachlink/src/types.rs` or create a new logging module:

```rust
// In types.rs or new logging.rs
#[macro_export]
macro_rules! log_and_panic {
    ($env:expr, $error:expr, $($arg:tt)*) => {
        $env.log(&format!("[ERROR] {}", format!($($arg)*)));
        panic_with_error!($env, $error)
    };
}
```

#### Update Error Handling
Replace all `panic_with_error!` calls with the new macro:

In `mobile_platform.rs`:
```rust
// Before
.unwrap_or_else(|| panic_with_error!(env, MobilePlatformError::DeviceNotSupported))

// After
.unwrap_or_else(|| log_and_panic!(env, MobilePlatformError::DeviceNotSupported, "Mobile profile not found for user"))
```

Apply similar changes to:
- `learning_paths.rs` line 740
- `content_quality.rs` line 1088
- `advanced_reputation.rs` line 669

#### Add Contextual Logging
Add Env.log() calls for key operations:
- Function entry/exit with parameters (at debug level)
- Important state changes
- External calls
- Validation failures

### 2. NestJS Indexer Logging Enhancement

#### Install Dependencies
```bash
cd indexer
npm install pino-http nestjs-pino
```

#### Configure Logger
Update `indexer/src/main.ts`:

```typescript
import { NestFactory } from '@nestjs/core';
import { Logger } from 'nestjs-pino';
import { AppModule } from './app.module';

async function bootstrap() {
  const app = await NestFactory.create(AppModule, {
    bufferLogs: true,
  });
  
  // Use Pino logger
  app.useLogger(app.get(Logger));
  
  await app.listen(process.env.PORT ?? 3000);
}
bootstrap();
```

#### Add Correlation ID Middleware
Create `indexer/src/middleware/correlation-id.middleware.ts`:

```typescript
import { Injectable, NestMiddleware } from '@nestjs/common';
import { Request, Response, NextFunction } from 'express';

@Injectable()
export class CorrelationIdMiddleware implements NestMiddleware {
  use(req: Request, res: Response, next: NextFunction) {
    const correlationId = req.headers['x-correlation-id'] || 
                         req.get('X-Correlation-ID') ||
                         this.generateId();
    
    req['correlationId'] = correlationId;
    res.setHeader('X-Correlation-ID', correlationId);
    
    next();
  }
  
  private generateId(): string {
    return Math.random().toString(36).substring(2, 15) +
           Math.random().toString(36).substring(2, 15);
  }
}
```

Register middleware in `app.module.ts`:
```typescript
import { MiddlewareConsumer, Module, NestModule } from '@nestjs/common';
import { CorrelationIdMiddleware } from './middleware/correlation-id.middleware';

@Module({
  // ... existing imports
})
export class AppModule implements NestModule {
  configure(consumer: MiddlewareConsumer) {
    consumer
      .apply(CorrelationIdMiddleware)
      .forRoutes('*');
  }
}
```

#### Update Services to Use Structured Logging
Enhance existing logger usage to include correlation IDs:

In services like `event-processor.service.ts`:
```typescript
// Before
this.logger.debug(`Processing event type: ${eventType}`);

// After
this.logger.debug(`Processing event type: ${eventType}`, {
  correlationId: this.requestId ?? 'unknown',
  eventType,
  // Add other relevant context
});
```

#### Configure JSON Logging and Log Levels
Add to `indexer/src/config/configuration.ts`:
```typescript
export default () => ({
  // ... existing config
  logging: {
    level: process.env.LOG_LEVEL || 'info',
    format: process.env.LOG_FORMAT || 'json',
  },
});
```

Update logger bootstrap to use configuration.

### 3. Log Aggregation Configuration

#### Update docker-compose.yml
Add logging drivers and options:
```yaml
services:
  indexer:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

#### Add Log Rotation Configuration
Create logging configuration files if needed for production deployment.

### 4. Documentation

#### Create LOGGING_STANDARDS.md
Document:
- Log format specification
- Correlation ID usage
- Log levels (DEBUG, INFO, WARN, ERROR)
- Field names and meanings
- Examples of structured logs
- How to correlate logs across services

### 5. Testing and Verification

#### Verify Logging Works
- Test that logs are generated correctly
- Verify correlation IDs propagate
- Check log levels can be configured
- Ensure no breaking changes to existing functionality

## Acceptance Criteria Checklist

- [ ] Add structured logging to all services (Rust contracts and NestJS)
- [ ] Implement correlation IDs for tracing
- [ ] Configure log aggregation capabilities
- [ ] Set up log level monitoring via environment variables
- [ ] Document logging standards in LOGGING_STANDARDS.md
- [ ] Replace all panic_with_error! calls with logging + panic
- [ ] Enhance NestJS services with structured logging context
- [ ] Verify logs are JSON formatted when configured
- [ ] Ensure correlation IDs appear in all log entries

## Files to Modify

### Rust Contracts
- `contracts/teachlink/src/types.rs` (add logging macro)
- `contracts/teachlink/src/mobile_platform.rs` (update panic call)
- `contracts/teachlink/src/learning_paths.rs` (update panic call)
- `contracts/teachlink/src/content_quality.rs` (update panic call)
- `contracts/teachlink/src/advanced_reputation.rs` (update panic call)
- Various other service files (add contextual logging)

### NestJS Indexer
- `indexer/package.json` (add dependencies)
- `indexer/src/main.ts` (configure pino logger)
- `indexer/src/middleware/correlation-id.middleware.ts` (new file)
- `indexer/src/app.module.ts` (register middleware)
- `indexer/src/config/configuration.ts` (add logging config)
- Various service files (enhance logging usage)

### Documentation
- `LOGGING_STANDARDS.md` (new file)

## Estimated Effort
- Rust logging macro and updates: 2 hours
- NestJS logging enhancement: 3 hours
- Documentation: 1 hour
- Testing and verification: 2 hours