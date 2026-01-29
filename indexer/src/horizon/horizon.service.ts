import { Injectable, Logger, OnModuleInit } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import * as StellarSdk from '@stellar/stellar-sdk';
import { Server, ServerApi } from '@stellar/stellar-sdk/lib/horizon';

export interface ProcessedEvent {
  type: string;
  data: any;
  ledger: string;
  txHash: string;
  timestamp: string;
  contractId: string;
}

@Injectable()
export class HorizonService implements OnModuleInit {
  private readonly logger = new Logger(HorizonService.name);
  private server: Server;
  private contractId: string;
  private networkPassphrase: string;

  constructor(private configService: ConfigService) {}

  async onModuleInit() {
    const horizonUrl = this.configService.get<string>('stellar.horizonUrl') || 'https://horizon-testnet.stellar.org';
    const network = this.configService.get<string>('stellar.network') || 'testnet';
    this.contractId = this.configService.get<string>('contract.teachlinkContractId') || '';

    this.server = new StellarSdk.Horizon.Server(horizonUrl);

    // Set network passphrase
    if (network === 'mainnet') {
      this.networkPassphrase = StellarSdk.Networks.PUBLIC;
    } else {
      this.networkPassphrase = StellarSdk.Networks.TESTNET;
    }

    this.logger.log(`Horizon service initialized for ${network} network`);
    this.logger.log(`Horizon URL: ${horizonUrl}`);
    this.logger.log(`Contract ID: ${this.contractId}`);
  }

  /**
   * Stream operations for a specific contract
   */
  async streamContractEvents(
    startLedger: string,
    onEvent: (event: ProcessedEvent) => Promise<void>,
    onError?: (error: Error) => void,
  ): Promise<() => void> {
    this.logger.log(`Starting event stream from ledger ${startLedger}`);

    let cursor = startLedger === 'latest' ? 'now' : startLedger;

    const closeHandler = this.server
      .operations()
      .cursor(cursor)
      .stream({
        onmessage: async (operation: any) => {
          try {
            // Only process invoke host function operations
            if (operation.type === 'invoke_host_function') {
              const invokeOp = operation as ServerApi.InvokeHostFunctionOperationRecord;

              // Check if this operation is for our contract
              if (this.isContractOperation(invokeOp)) {
                const events = await this.extractEventsFromOperation(invokeOp);

                for (const event of events) {
                  await onEvent(event);
                }
              }
            }
          } catch (error: any) {
            this.logger.error(`Error processing operation: ${error.message}`, error.stack);
            if (onError) {
              onError(error);
            }
          }
        },
        onerror: (error: any) => {
          this.logger.error(`Stream error: ${error.message}`, error.stack);
          if (onError) {
            onError(new Error(error.message || 'Stream error'));
          }
        },
      });

    return closeHandler;
  }

  /**
   * Fetch operations for a specific ledger range
   */
  async fetchOperationsInRange(
    startLedger: number,
    endLedger: number,
  ): Promise<ProcessedEvent[]> {
    this.logger.log(`Fetching operations from ledger ${startLedger} to ${endLedger}`);

    const allEvents: ProcessedEvent[] = [];

    for (let ledger = startLedger; ledger <= endLedger; ledger++) {
      try {
        const operations = await this.server
          .operations()
          .forLedger(ledger.toString())
          .limit(200)
          .call();

        for (const operation of operations.records) {
          if (operation.type === 'invoke_host_function') {
            const invokeOp = operation as ServerApi.InvokeHostFunctionOperationRecord;

            if (this.isContractOperation(invokeOp)) {
              const events = await this.extractEventsFromOperation(invokeOp);
              allEvents.push(...events);
            }
          }
        }
      } catch (error) {
        this.logger.warn(`Error fetching ledger ${ledger}: ${error.message}`);
      }
    }

    return allEvents;
  }

  /**
   * Get the latest ledger number
   */
  async getLatestLedger(): Promise<number> {
    const ledger = await this.server.ledgers().order('desc').limit(1).call();
    return ledger.records[0].sequence;
  }

  /**
   * Get a specific transaction
   */
  async getTransaction(txHash: string): Promise<ServerApi.TransactionRecord> {
    return this.server.transactions().transaction(txHash).call();
  }

  /**
   * Check if an operation is for our contract
   */
  private isContractOperation(operation: ServerApi.InvokeHostFunctionOperationRecord): boolean {
    // For Soroban contracts, we need to check the function parameter
    // This is a simplified check - in production, you'd parse the XDR more thoroughly
    return operation.function === 'HostFunctionTypeHostFunctionTypeInvokeContract';
  }

  /**
   * Extract events from a contract operation
   */
  private async extractEventsFromOperation(
    operation: ServerApi.InvokeHostFunctionOperationRecord,
  ): Promise<ProcessedEvent[]> {
    const events: ProcessedEvent[] = [];

    try {
      // Fetch the transaction to get events
      const tx = await this.getTransaction(operation.transaction_hash);

      // In Stellar SDK, contract events are stored in the transaction result meta
      // This would require parsing the XDR data
      // For this implementation, we'll use a simplified approach

      // Note: In production, you'd need to:
      // 1. Parse the transaction result meta XDR
      // 2. Extract contract events from the meta
      // 3. Decode the event data using the contract's event schema

      const processedEvent: ProcessedEvent = {
        type: 'ContractEvent',
        data: {}, // Would contain decoded event data
        ledger: (operation as any).ledger?.toString() || '0',
        txHash: operation.transaction_hash,
        timestamp: operation.created_at,
        contractId: this.contractId,
      };

      events.push(processedEvent);
    } catch (error) {
      this.logger.error(`Error extracting events: ${error.message}`);
    }

    return events;
  }

  /**
   * Parse Soroban event from XDR
   * This is a placeholder - actual implementation would parse XDR
   */
  private parseContractEvent(eventXdr: any): ProcessedEvent | null {
    // TODO: Implement proper XDR parsing
    // This would involve:
    // 1. Decoding the XDR event data
    // 2. Matching event topics to known event types
    // 3. Extracting event data fields
    return null;
  }
}
