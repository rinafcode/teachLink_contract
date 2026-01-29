export default () => ({
  stellar: {
    network: process.env.STELLAR_NETWORK || 'testnet',
    horizonUrl: process.env.HORIZON_URL || 'https://horizon-testnet.stellar.org',
    sorobanRpcUrl: process.env.SOROBAN_RPC_URL || 'https://soroban-testnet.stellar.org',
  },
  contract: {
    teachlinkContractId: process.env.TEACHLINK_CONTRACT_ID,
  },
  database: {
    type: process.env.DB_TYPE || 'postgres',
    host: process.env.DB_HOST || 'localhost',
    port: parseInt(process.env.DB_PORT, 10) || 5432,
    username: process.env.DB_USERNAME || 'teachlink',
    password: process.env.DB_PASSWORD,
    database: process.env.DB_DATABASE || 'teachlink_indexer',
    synchronize: process.env.DB_SYNCHRONIZE === 'true',
    logging: process.env.DB_LOGGING === 'true',
  },
  indexer: {
    pollInterval: parseInt(process.env.INDEXER_POLL_INTERVAL, 10) || 5000,
    startLedger: process.env.INDEXER_START_LEDGER || 'latest',
    batchSize: parseInt(process.env.INDEXER_BATCH_SIZE, 10) || 100,
  },
  app: {
    nodeEnv: process.env.NODE_ENV || 'development',
    port: parseInt(process.env.PORT, 10) || 3000,
    logLevel: process.env.LOG_LEVEL || 'debug',
  },
});
