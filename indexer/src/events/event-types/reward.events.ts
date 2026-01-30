export interface RewardIssuedEvent {
  recipient: string;
  amount: string;
  reward_type: string;
  timestamp: string;
}

export interface RewardClaimedEvent {
  user: string;
  amount: string;
  timestamp: string;
}

export interface RewardPoolFundedEvent {
  funder: string;
  amount: string;
  timestamp: string;
}

export type RewardEvent =
  | { type: 'RewardIssuedEvent'; data: RewardIssuedEvent }
  | { type: 'RewardClaimedEvent'; data: RewardClaimedEvent }
  | { type: 'RewardPoolFundedEvent'; data: RewardPoolFundedEvent };
