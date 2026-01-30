export interface EscrowCreatedEvent {
  escrow: {
    id: string;
    depositor: string;
    beneficiary: string;
    amount: string;
    required_signers: string[];
    required_approvals: number;
    deadline?: string;
  };
}

export interface EscrowApprovedEvent {
  escrow_id: string;
  signer: string;
  approval_count: number;
}

export interface EscrowReleasedEvent {
  escrow_id: string;
  beneficiary: string;
  amount: string;
}

export interface EscrowRefundedEvent {
  escrow_id: string;
  depositor: string;
  amount: string;
}

export interface EscrowDisputedEvent {
  escrow_id: string;
  disputer: string;
  reason: string;
}

export interface EscrowResolvedEvent {
  escrow_id: string;
  outcome: string;
  status: string;
}

export type EscrowEvent =
  | { type: 'EscrowCreatedEvent'; data: EscrowCreatedEvent }
  | { type: 'EscrowApprovedEvent'; data: EscrowApprovedEvent }
  | { type: 'EscrowReleasedEvent'; data: EscrowReleasedEvent }
  | { type: 'EscrowRefundedEvent'; data: EscrowRefundedEvent }
  | { type: 'EscrowDisputedEvent'; data: EscrowDisputedEvent }
  | { type: 'EscrowResolvedEvent'; data: EscrowResolvedEvent };
