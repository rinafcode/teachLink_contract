# TeachLink Indexer

A real-time blockchain indexer for TeachLink Soroban smart contracts built with NestJS and Stellar Horizon API.

## Overview

The TeachLink Indexer monitors the Stellar blockchain for TeachLink contract events and indexes them into a PostgreSQL database, enabling efficient querying and analytics of on-chain data.

## Features

- **Real-time Event Monitoring**: Continuously streams events from Stellar Horizon API
- **Comprehensive Event Coverage**: Indexes all 18+ TeachLink contract event types
- **Persistent State**: Tracks indexing progress with automatic resume capability
- **Historical Backfill**: Support for indexing historical blockchain data
- **Health Monitoring**: Built-in health checks and error tracking
- **Type-Safe**: Full TypeScript implementation with comprehensive type definitions
- **Well-Tested**: Extensive unit and integration test coverage
- **Production-Ready**: Docker support with multi-stage builds

## Architecture

### Components

#### 1. Horizon Service ([horizon.service.ts](src/horizon/horizon.service.ts))
- Interfaces with Stellar Horizon API
- Streams real-time blockchain operations
- Fetches historical ledger data
- Parses Soroban contract events

#### 2. Event Processor ([event-processor.service.ts](src/events/event-processor.service.ts))
- Processes 18+ event types from TeachLink contracts
- Transforms blockchain events into database entities
- Handles event-specific business logic

#### 3. Indexer Service ([indexer.service.ts](src/indexer/indexer.service.ts))
- Orchestrates the indexing process
- Manages indexer lifecycle and state
- Implements health checks and error recovery
- Supports historical data backfill

#### 4. Database Layer
- TypeORM entities for all contract data types
- PostgreSQL for persistent storage
- Indexed columns for optimized queries

### Indexed Events

**Bridge Events:**
- DepositEvent
- ReleaseEvent
- BridgeInitiatedEvent
- BridgeCompletedEvent

**Reward Events:**
- RewardIssuedEvent
- RewardClaimedEvent
- RewardPoolFundedEvent

**Escrow Events:**
- EscrowCreatedEvent
- EscrowApprovedEvent
- EscrowReleasedEvent
- EscrowRefundedEvent
- EscrowDisputedEvent
- EscrowResolvedEvent

**Tokenization Events:**
- ContentMintedEvent
- OwnershipTransferredEvent
- ProvenanceRecordedEvent
- MetadataUpdatedEvent

**Scoring Events:**
- CreditScoreUpdatedEvent
- CourseCompletedEvent
- ContributionRecordedEvent

## Installation

### Prerequisites

- Node.js 20+
- PostgreSQL 16+
- Docker & Docker Compose (optional)

### Local Setup

1. Clone the repository:
```bash
cd indexer
```

2. Install dependencies:
```bash
npm install
```

3. Configure environment:
```bash
cp .env.example .env
# Edit .env with your configuration
```

4. Set up the database:
```bash
# Create PostgreSQL database
createdb teachlink_indexer

# Run migrations (auto-sync enabled in development)
npm run start:dev
```

### Docker Setup

1. Configure environment:
```bash
cp .env.example .env
# Edit .env with your configuration
```

2. Start services:
```bash
# Development mode
docker-compose up indexer

# Production mode
docker-compose --profile production up indexer-prod
```

## Configuration

Configure the indexer via environment variables in [.env](.env.example):

### Stellar Network
- `STELLAR_NETWORK`: Network to use (testnet, mainnet)
- `HORIZON_URL`: Horizon API endpoint
- `SOROBAN_RPC_URL`: Soroban RPC endpoint

### Contract
- `TEACHLINK_CONTRACT_ID`: TeachLink contract address

### Database
- `DB_TYPE`: Database type (postgres)
- `DB_HOST`: Database host
- `DB_PORT`: Database port
- `DB_USERNAME`: Database username
- `DB_PASSWORD`: Database password
- `DB_DATABASE`: Database name
- `DB_SYNCHRONIZE`: Auto-sync schema (true for dev, false for prod)
- `DB_LOGGING`: Enable SQL logging

### Indexer
- `INDEXER_POLL_INTERVAL`: Polling interval in ms (default: 5000)
- `INDEXER_START_LEDGER`: Starting ledger (latest or specific number)
- `INDEXER_BATCH_SIZE`: Batch size for backfill (default: 100)

## Usage

### Development

```bash
# Start in development mode with hot reload
npm run start:dev

# Run tests
npm run test

# Run integration tests
npm run test:e2e

# Generate test coverage
npm run test:cov

# Lint code
npm run lint

# Format code
npm run format
```

### Production

```bash
# Build the application
npm run build

# Start in production mode
npm run start:prod
```

### Docker

```bash
# Development
docker-compose up indexer

# Production
docker-compose --profile production up indexer-prod

# View logs
docker-compose logs -f indexer

# Stop services
docker-compose down
```

## Database Schema

### Core Tables

- `bridge_transactions`: Cross-chain bridge operations
- `rewards`: Reward issuance and claims
- `escrows`: Multi-signature escrow records
- `content_tokens`: Educational content NFTs
- `provenance_records`: Token ownership history
- `credit_scores`: User credit scores
- `course_completions`: Course completion records
- `contributions`: User contribution tracking
- `reward_pool`: Reward pool state
- `indexer_state`: Indexer progress tracking

All tables include proper indexes for efficient querying.

## API

The indexer exposes the following programmatic interfaces:

### IndexerService

```typescript
// Get current indexer status
const status = await indexerService.getStatus();

// Backfill historical data
await indexerService.backfillHistoricalData(startLedger, endLedger);

// Start/stop indexing
await indexerService.startIndexing();
await indexerService.stopIndexing();
```

## Testing

### Unit Tests

```bash
npm run test
```

Test coverage includes:
- Horizon service event streaming
- Event processor for all event types
- Indexer service lifecycle
- Database entity operations

### Integration Tests

```bash
npm run test:e2e
```

Integration tests verify:
- End-to-end indexing flow
- Database schema creation
- Service initialization

### Test Coverage

```bash
npm run test:cov
```

## Monitoring

### Health Checks

The indexer includes automatic health monitoring:
- Runs every 5 minutes
- Checks if indexer is running
- Auto-restarts on failure
- Logs status and metrics

### Metrics

Track indexer performance via the `indexer_state` table:
- `lastProcessedLedger`: Most recent indexed ledger
- `totalEventsProcessed`: Total events indexed
- `totalErrors`: Total errors encountered
- `updatedAt`: Last update timestamp

## Troubleshooting

### Indexer Not Starting

1. Check database connection:
```bash
psql -h localhost -U teachlink -d teachlink_indexer
```

2. Verify contract ID is set:
```bash
echo $TEACHLINK_CONTRACT_ID
```

3. Check Horizon API connectivity:
```bash
curl https://horizon-testnet.stellar.org
```

### Missing Events

1. Check indexer status:
```typescript
const status = await indexerService.getStatus();
console.log(status);
```

2. Backfill missing ledgers:
```typescript
await indexerService.backfillHistoricalData(startLedger, endLedger);
```

### Performance Issues

1. Increase batch size:
```bash
INDEXER_BATCH_SIZE=200
```

2. Add database indexes (already configured)

3. Scale horizontally with multiple indexers (advanced)

## Development

### Project Structure

```
indexer/
├── src/
│   ├── config/              # Configuration
│   ├── database/            # Database entities & module
│   │   └── entities/        # TypeORM entities
│   ├── events/              # Event processing
│   │   └── event-types/     # Event type definitions
│   ├── horizon/             # Horizon API integration
│   ├── indexer/             # Main indexer service
│   ├── app.module.ts        # Root module
│   └── main.ts              # Application entry point
├── test/                    # Integration tests
├── docker-compose.yml       # Docker services
├── Dockerfile               # Multi-stage build
└── package.json             # Dependencies & scripts
```

### Adding New Event Types

1. Define event type in [src/events/event-types/](src/events/event-types/)
2. Create database entity in [src/database/entities/](src/database/entities/)
3. Add event handler in [event-processor.service.ts](src/events/event-processor.service.ts)
4. Write tests

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Write/update tests
5. Run linting and tests
6. Submit a pull request

## License

MIT

## Support

For issues and questions:
- Create an issue in the repository
- Check existing documentation
- Review test files for usage examples
