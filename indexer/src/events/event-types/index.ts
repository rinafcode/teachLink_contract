export * from './bridge.events';
export * from './reward.events';
export * from './escrow.events';
export * from './tokenization.events';
export * from './scoring.events';

import { BridgeEvent } from './bridge.events';
import { RewardEvent } from './reward.events';
import { EscrowEvent } from './escrow.events';
import { TokenizationEvent } from './tokenization.events';
import { ScoringEvent } from './scoring.events';

export type ContractEvent =
  | BridgeEvent
  | RewardEvent
  | EscrowEvent
  | TokenizationEvent
  | ScoringEvent;
