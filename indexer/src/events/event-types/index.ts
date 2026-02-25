export * from './bridge.events';
export * from './reward.events';
export * from './escrow.events';
export * from './tokenization.events';
export * from './scoring.events';
export * from './reporting.events';
export * from './backup.events';

import { BridgeEvent } from './bridge.events';
import { RewardEvent } from './reward.events';
import { EscrowEvent } from './escrow.events';
import { TokenizationEvent } from './tokenization.events';
import { ScoringEvent } from './scoring.events';
import { ReportingEvent } from './reporting.events';
import { BackupEvent } from './backup.events';

export type ContractEvent =
  | BridgeEvent
  | RewardEvent
  | EscrowEvent
  | TokenizationEvent
  | ScoringEvent
  | ReportingEvent
  | BackupEvent;
