import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';

/**
 * Validated, typed snapshot of all indexer configuration.
 *
 * Every field has a documented source env-var and default value.
 * Validation runs at startup and on every hot-reload.
 */
export interface IndexerConfig {
  /** Stellar network name. Env: STELLAR_NETWORK. Default: testnet */
  stellarNetwork: string;
  /** Horizon base URL. Env: HORIZON_URL */
  horizonUrl: string;
  /** Soroban RPC URL. Env: SOROBAN_RPC_URL */
  sorobanRpcUrl: string;
  /** TeachLink contract ID. Env: TEACHLINK_CONTRACT_ID */
  teachlinkContractId: string;

  /** PostgreSQL host. Env: DB_HOST. Default: localhost */
  dbHost: string;
  /** PostgreSQL port. Env: DB_PORT. Default: 5432 */
  dbPort: number;
  /** PostgreSQL username. Env: DB_USERNAME. Default: teachlink */
  dbUsername: string;
  /** PostgreSQL password. Env: DB_PASSWORD */
  dbPassword: string;
  /** PostgreSQL database name. Env: DB_DATABASE. Default: teachlink_indexer */
  dbDatabase: string;
  /** Auto-synchronize schema. Env: DB_SYNCHRONIZE. Default: false */
  dbSynchronize: boolean;
  /** Enable query logging. Env: DB_LOGGING. Default: false */
  dbLogging: boolean;

  /** Event poll interval (ms). Env: INDEXER_POLL_INTERVAL. Default: 5000 */
  pollIntervalMs: number;
  /** Starting ledger. Env: INDEXER_START_LEDGER. Default: latest */
  startLedger: string;
  /** Events per batch. Env: INDEXER_BATCH_SIZE. Default: 100 */
  batchSize: number;
  /** Seconds before indexer is considered stale. Env: INDEXER_STALE_AFTER_SECONDS. Default: 900 */
  staleAfterSeconds: number;

  /** Node environment. Env: NODE_ENV. Default: development */
  nodeEnv: string;
  /** HTTP port. Env: PORT. Default: 3000 */
  port: number;
  /** Log level. Env: LOG_LEVEL. Default: debug */
  logLevel: string;
}

/** Validation errors collected during config load. */
export interface ConfigValidationError {
  field: string;
  message: string;
}

/**
 * ConfigManager centralizes all configuration access, validates values at
 * startup, and supports hot-reload by re-reading from ConfigService on demand.
 *
 * ## Hot-reload
 * Call `reload()` to re-validate and refresh the in-memory snapshot without
 * restarting the process. Useful when env vars are updated at runtime (e.g.
 * via a secrets manager sidecar).
 *
 * ## Validation rules
 * - `horizonUrl` and `sorobanRpcUrl` must be non-empty strings starting with http
 * - `dbPort` must be 1–65535
 * - `pollIntervalMs` must be >= 1000 ms
 * - `batchSize` must be 1–10000
 * - `port` must be 1–65535
 */
@Injectable()
export class ConfigManager implements OnModuleInit {
  private readonly logger = new Logger(ConfigManager.name);
  private snapshot: IndexerConfig;

  constructor(private readonly configService: ConfigService) {}

  onModuleInit(): void {
    this.snapshot = this.load();
    const errors = this.validate(this.snapshot);
    if (errors.length > 0) {
      for (const e of errors) {
        this.logger.error(`Config validation failed [${e.field}]: ${e.message}`);
      }
      throw new Error(
        `Configuration is invalid. Fix the following: ${errors.map((e) => e.field).join(', ')}`,
      );
    }
    this.logger.log(
      `Configuration loaded (network=${this.snapshot.stellarNetwork}, env=${this.snapshot.nodeEnv})`,
    );
  }

  /**
   * Re-read all values from ConfigService, validate, and update the snapshot.
   * Returns validation errors if any; an empty array means success.
   */
  reload(): ConfigValidationError[] {
    const next = this.load();
    const errors = this.validate(next);
    if (errors.length === 0) {
      this.snapshot = next;
      this.logger.log('Configuration hot-reloaded successfully');
    } else {
      this.logger.warn(
        `Hot-reload aborted — validation errors: ${errors.map((e) => e.field).join(', ')}`,
      );
    }
    return errors;
  }

  /** Return the current validated configuration snapshot. */
  get(): IndexerConfig {
    return this.snapshot;
  }

  // ===== Private =====

  private load(): IndexerConfig {
    return {
      stellarNetwork: this.configService.get<string>('stellar.network', 'testnet'),
      horizonUrl: this.configService.get<string>('stellar.horizonUrl', ''),
      sorobanRpcUrl: this.configService.get<string>('stellar.sorobanRpcUrl', ''),
      teachlinkContractId: this.configService.get<string>('contract.teachlinkContractId', ''),

      dbHost: this.configService.get<string>('database.host', 'localhost'),
      dbPort: this.configService.get<number>('database.port', 5432),
      dbUsername: this.configService.get<string>('database.username', 'teachlink'),
      dbPassword: this.configService.get<string>('database.password', ''),
      dbDatabase: this.configService.get<string>('database.database', 'teachlink_indexer'),
      dbSynchronize: this.configService.get<boolean>('database.synchronize', false),
      dbLogging: this.configService.get<boolean>('database.logging', false),

      pollIntervalMs: this.configService.get<number>('indexer.pollInterval', 5000),
      startLedger: this.configService.get<string>('indexer.startLedger', 'latest'),
      batchSize: this.configService.get<number>('indexer.batchSize', 100),
      staleAfterSeconds: this.configService.get<number>('indexer.staleAfterSeconds', 900),

      nodeEnv: this.configService.get<string>('app.nodeEnv', 'development'),
      port: this.configService.get<number>('app.port', 3000),
      logLevel: this.configService.get<string>('app.logLevel', 'debug'),
    };
  }

  private validate(c: IndexerConfig): ConfigValidationError[] {
    const errors: ConfigValidationError[] = [];

    if (!c.horizonUrl.startsWith('http')) {
      errors.push({ field: 'horizonUrl', message: 'Must be a valid http(s) URL' });
    }
    if (!c.sorobanRpcUrl.startsWith('http')) {
      errors.push({ field: 'sorobanRpcUrl', message: 'Must be a valid http(s) URL' });
    }
    if (c.dbPort < 1 || c.dbPort > 65535) {
      errors.push({ field: 'dbPort', message: 'Must be between 1 and 65535' });
    }
    if (!c.dbPassword) {
      errors.push({ field: 'dbPassword', message: 'DB_PASSWORD must be set' });
    }
    if (c.pollIntervalMs < 1000) {
      errors.push({ field: 'pollIntervalMs', message: 'Must be >= 1000 ms' });
    }
    if (c.batchSize < 1 || c.batchSize > 10000) {
      errors.push({ field: 'batchSize', message: 'Must be between 1 and 10000' });
    }
    if (c.port < 1 || c.port > 65535) {
      errors.push({ field: 'port', message: 'Must be between 1 and 65535' });
    }

    return errors;
  }
}
