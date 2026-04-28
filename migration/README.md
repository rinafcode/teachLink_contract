# TeachLink Contract Migration Tools

Comprehensive migration tools for TeachLink smart contracts with automation, validation, rollback, and progress tracking capabilities.

## Overview

This migration toolkit provides a complete solution for upgrading TeachLink contracts across different networks while ensuring safety, reliability, and easy recovery from failures.

## Features

- **Automated Migration**: Streamlined migration process with configurable steps
- **Comprehensive Validation**: Pre and post-migration validation checks
- **Rollback Support**: Automatic and manual rollback capabilities
- **Progress Tracking**: Real-time monitoring and reporting
- **Multi-Network Support**: Works with testnet, mainnet, and local networks
- **Backup & Recovery**: Automatic state backups with integrity verification

## Directory Structure

```
migration/
├── migrate.sh          # Main migration automation script
├── validate.sh         # Validation and health checks
├── rollback.sh         # Rollback functionality
├── progress.sh         # Progress tracking and reporting
├── config_template.json # Migration configuration template
├── scripts/            # Custom migration scripts
├── logs/               # Migration logs
├── reports/            # Generated reports
└── backups/            # Contract state backups
```

## Quick Start

### 1. Prepare Migration

```bash
# Validate environment and contract
./migration/validate.sh --network testnet --contract-id CB4HK... --identity deployer --type pre

# Check current migration status
./migration/progress.sh status --network testnet
```

### 2. Execute Migration

```bash
# Run automated migration
./migration/migrate.sh \
  --network testnet \
  --identity deployer \
  --contract-id CB4HK... \
  --new-wasm ./target/wasm32-unknown-unknown/release/teachlink_contract_v2.wasm \
  --config ./migration/config_template.json
```

### 3. Monitor Progress

```bash
# Monitor migration in real-time
./migration/progress.sh monitor --contract-id CB4HK... --follow

# Generate migration report
./migration/progress.sh report --format html > migration_report.html
```

### 4. Validation

```bash
# Run post-migration validation
./migration/validate.sh --contract-id CB4HK... --type post --format json
```

## Detailed Usage

### Migration Script (`migrate.sh`)

Automates the complete migration process with validation and error handling.

```bash
./migration/migrate.sh [options]

Options:
  --network <name>           Network to migrate on
  --identity <name>          Deployer identity name
  --contract-id <id>         Current contract ID
  --new-wasm <path>          Path to new contract WASM
  --config <path>            Migration configuration file
  --dry-run                  Preview commands without execution
  --non-interactive          Skip confirmations
  --skip-validation          Skip validation checks
  --force                    Force migration
  --no-rollback              Disable automatic rollback on failure
```

**Example Migration:**

```bash
./migration/migrate.sh \
  --network mainnet \
  --identity teachlink-admin \
  --contract-id CB4HKABCD123... \
  --new-wasm ./contracts/teachlink/target/wasm32-unknown-unknown/release/teachlink_contract.wasm \
  --config ./migration/v2_migration_config.json
```

### Validation Script (`validate.sh`)

Performs comprehensive validation checks before and after migration.

```bash
./migration/validate.sh [options]

Options:
  --network <name>       Network to validate on
  --identity <name>      Identity for validation
  --contract-id <id>     Contract ID to validate
  --type <type>          Validation type: pre|post|full
  --format <format>      Output format: text|json
  --verbose              Enable verbose output
```

**Validation Checks:**

- Environment setup (CLI, network config)
- Network connectivity and health
- Contract connectivity and responsiveness
- Contract version verification
- Admin access validation
- Contract state integrity
- Gas estimation for operations
- Rollback availability (post-migration)

### Rollback Script (`rollback.sh`)

Provides rollback functionality for failed migrations.

```bash
./migration/rollback.sh [options]

Options:
  --network <name>       Network to rollback on
  --identity <name>      Identity for rollback
  --contract-id <id>     Contract ID to rollback
  --type <type>          Rollback type: auto|manual|emergency
  --dry-run              Preview commands
  --non-interactive      Skip confirmations
  --force                Force rollback
```

**Rollback Types:**

- **Auto**: Uses contract's built-in rollback function
- **Manual**: Custom rollback with specific steps
- **Emergency**: Destructive rollback for critical failures

### Progress Tracking (`progress.sh`)

Monitors migration progress and generates reports.

```bash
./migration/progress.sh <command> [options]

Commands:
  status     Show current migration status
  monitor    Monitor ongoing migration
  report     Generate migration report
  cleanup    Clean up old artifacts

Options:
  --network <name>       Network name
  --contract-id <id>     Contract ID
  --follow               Follow mode for monitoring
  --format <format>      Report format: text|json|html
```

## Migration Configuration

Create a JSON configuration file for complex migrations:

```json
{
  "name": "TeachLink v2.0 Migration",
  "target_version": 2,
  "migration_script": "./migration/scripts/custom_migration.sh",
  "validation_checks": ["contract_connectivity", "admin_access"],
  "rollback_timeout": 2592000,
  "backup_data": true,
  "pre_migration_steps": ["pause_operations"],
  "post_migration_steps": ["verify_functionality"]
}
```

## Safety Features

### Automatic Backups

- Contract state snapshots before migration
- WASM file backups
- Configuration and metadata preservation

### Validation Checks

- Pre-migration environment validation
- Post-migration functionality verification
- Gas estimation and network health checks

### Rollback Protection

- Time-limited rollback windows
- State integrity verification
- Automatic rollback on migration failure

### Progress Tracking

- Step-by-step progress logging
- Real-time monitoring capabilities
- Comprehensive reporting

## Best Practices

### Pre-Migration

1. **Test Thoroughly**: Run migrations on testnet first
2. **Backup Everything**: Ensure all data is backed up
3. **Validate Access**: Confirm admin rights and network access
4. **Check Gas**: Estimate and fund accounts adequately
5. **Notify Stakeholders**: Inform users of potential downtime

### During Migration

1. **Monitor Closely**: Use progress tracking tools
2. **Have Rollback Ready**: Keep rollback scripts prepared
3. **Log Everything**: Maintain detailed migration logs
4. **Be Patient**: Allow time for network confirmations

### Post-Migration

1. **Validate Immediately**: Run post-migration checks
2. **Test Functionality**: Verify all contract features work
3. **Update Clients**: Ensure applications use new contract
4. **Monitor Health**: Watch for issues in production
5. **Document Results**: Record migration outcomes

## Troubleshooting

### Common Issues

**Migration Fails During Preparation:**

- Check contract admin permissions
- Verify network connectivity
- Ensure sufficient gas funding

**Validation Errors:**

- Confirm contract ID is correct
- Check network configuration
- Verify CLI installation

**Rollback Unavailable:**

- Migration may be outside rollback window
- Contract may not support rollback
- State backup may be corrupted

### Recovery Steps

1. **Stop the Migration**: Cancel any ongoing processes
2. **Assess Damage**: Check contract state and functionality
3. **Execute Rollback**: Use appropriate rollback type
4. **Validate Recovery**: Confirm system is restored
5. **Investigate Root Cause**: Analyze logs for failure reasons
6. **Retry with Fixes**: Address issues and retry migration

## Integration with CI/CD

The migration tools can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
name: Contract Migration
on:
  push:
    tags:
      - "v*"

jobs:
  migrate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Soroban
        run: curl -L https://github.com/stellar/soroban-cli/releases/latest/download/soroban-cli-linux-amd64.tar.gz | tar xz
      - name: Validate
        run: ./migration/validate.sh --network testnet --contract-id ${{ secrets.CONTRACT_ID }} --type pre
      - name: Migrate
        run: ./migration/migrate.sh --network testnet --identity ${{ secrets.IDENTITY }} --contract-id ${{ secrets.CONTRACT_ID }} --new-wasm ./target/wasm32-unknown-unknown/release/contract.wasm
      - name: Report
        run: ./migration/progress.sh report --format html > migration_report.html
```

## Security Considerations

- **Access Control**: Limit migration execution to authorized personnel
- **Network Security**: Use secure networks for mainnet migrations
- **Key Management**: Protect private keys and identities
- **Audit Trail**: Maintain comprehensive logs for compliance
- **Testing**: Never migrate to mainnet without thorough testing

## Support

For issues or questions:

- Check the troubleshooting section above
- Review migration logs in `migration/logs/`
- Validate contract state manually
- Contact the development team

## Contributing

When adding new migration features:

1. Update this README with new functionality
2. Add validation checks for new features
3. Include rollback support for new operations
4. Update the configuration template
5. Test thoroughly on testnet before mainnet deployment
