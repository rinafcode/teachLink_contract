export interface DepositEvent {
  nonce: string;
  from: string;
  amount: string;
  destination_chain: string;
  destination_address: string;
}

export interface ReleaseEvent {
  nonce: string;
  recipient: string;
  amount: string;
  source_chain: string;
}

export interface BridgeInitiatedEvent {
  nonce: string;
  transaction: {
    from: string;
    amount: string;
    destination_chain: string;
    destination_address: string;
  };
}

export interface BridgeCompletedEvent {
  nonce: string;
  message: string;
}

export type BridgeEvent =
  | { type: 'DepositEvent'; data: DepositEvent }
  | { type: 'ReleaseEvent'; data: ReleaseEvent }
  | { type: 'BridgeInitiatedEvent'; data: BridgeInitiatedEvent }
  | { type: 'BridgeCompletedEvent'; data: BridgeCompletedEvent };
