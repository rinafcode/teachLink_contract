export interface BackupCreatedEvent {
  backup_id: string;
  created_by: string;
  integrity_hash: string;
  rto_tier: string;
  created_at: string;
}

export interface BackupVerifiedEvent {
  backup_id: string;
  verified_by: string;
  verified_at: string;
  valid: boolean;
}

export interface RecoveryExecutedEvent {
  recovery_id: string;
  backup_id: string;
  executed_by: string;
  recovery_duration_secs: string;
  success: boolean;
}

export type BackupEvent =
  | { type: 'BackupCreatedEvent'; data: BackupCreatedEvent }
  | { type: 'BackupVerifiedEvent'; data: BackupVerifiedEvent }
  | { type: 'RecoveryExecutedEvent'; data: RecoveryExecutedEvent };
