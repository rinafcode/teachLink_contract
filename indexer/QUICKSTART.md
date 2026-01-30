# TeachLink Indexer - Quick Start Guide

Get the TeachLink Indexer running in 5 minutes.

## Prerequisites

- Docker & Docker Compose
- TeachLink contract deployed on Stellar

## Quick Start (Docker)

### 1. Clone and Configure

```bash
cd indexer
cp .env.example .env
```

### 2. Edit Configuration

Edit `.env` and set your contract ID:

```env
TEACHLINK_CONTRACT_ID=your_contract_id_here
```

### 3. Start Services

```bash
# Start in development mode
docker-compose up indexer

# Or start in production mode
docker-compose --profile production up indexer-prod
```

That's it! The indexer is now running and monitoring the blockchain.

## Verify It's Working

### Check Logs

```bash
docker-compose logs -f indexer
```

You should see:
```
TeachLink Indexer is running on port 3000
Horizon service initialized for testnet network
Starting event stream from ledger...
```

### Check Database

```bash
# Connect to PostgreSQL
docker-compose exec postgres psql -U teachlink -d teachlink_indexer

# List tables
\dt

# Query indexer state
SELECT * FROM indexer_state;
```

## Next Steps

- [Read the full README](README.md) for detailed documentation
- [Check IMPLEMENTATION.md](IMPLEMENTATION.md) for architecture details
- Explore the indexed data in PostgreSQL
- Build applications on top of the indexed data

## Common Issues

### Contract ID Not Set

```
Error: TEACHLINK_CONTRACT_ID is not configured
```

**Solution:** Set `TEACHLINK_CONTRACT_ID` in `.env`

### Database Connection Failed

```
Error: Connection refused to postgres:5432
```

**Solution:** Wait for PostgreSQL to start:
```bash
docker-compose up -d postgres
# Wait 10 seconds
docker-compose up indexer
```

### No Events Detected

**Reasons:**
1. Contract not deployed
2. No activity on the contract
3. Wrong network (testnet vs mainnet)

**Solution:** Verify contract ID and network in `.env`

## Manual Setup (Without Docker)

### Prerequisites

- Node.js 20+
- PostgreSQL 16+

### Steps

```bash
# 1. Install dependencies
npm install

# 2. Configure environment
cp .env.example .env
# Edit .env with your settings

# 3. Create database
createdb teachlink_indexer

# 4. Start the indexer
npm run start:dev
```

## Development Workflow

```bash
# Run tests
npm run test

# Run integration tests
npm run test:e2e

# Lint code
npm run lint

# Format code
npm run format

# Build for production
npm run build
```

## Helpful Commands

```bash
# View all services
docker-compose ps

# Stop services
docker-compose down

# Remove volumes (reset database)
docker-compose down -v

# View real-time logs
docker-compose logs -f indexer

# Restart indexer
docker-compose restart indexer
```

## Production Deployment

For production deployment:

1. Use the production Docker Compose profile
2. Set `DB_SYNCHRONIZE=false`
3. Use strong passwords
4. Enable SSL for database connections
5. Set up monitoring and alerting
6. Configure backups

See [README.md](README.md) for full production setup guide.
