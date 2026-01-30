export interface ContentMintedEvent {
  token_id: string;
  creator: string;
  metadata: {
    content_hash: string;
    metadata_uri?: string;
    transferable: boolean;
    royalty_percentage: number;
  };
}

export interface OwnershipTransferredEvent {
  token_id: string;
  from: string;
  to: string;
  timestamp: string;
}

export interface ProvenanceRecordedEvent {
  token_id: string;
  record: {
    from?: string;
    to: string;
    timestamp: string;
    event_type: string;
  };
}

export interface MetadataUpdatedEvent {
  token_id: string;
  owner: string;
  timestamp: string;
}

export type TokenizationEvent =
  | { type: 'ContentMintedEvent'; data: ContentMintedEvent }
  | { type: 'OwnershipTransferredEvent'; data: OwnershipTransferredEvent }
  | { type: 'ProvenanceRecordedEvent'; data: ProvenanceRecordedEvent }
  | { type: 'MetadataUpdatedEvent'; data: MetadataUpdatedEvent };
