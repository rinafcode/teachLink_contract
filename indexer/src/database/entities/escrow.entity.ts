import { Entity, Column, PrimaryGeneratedColumn, Index, CreateDateColumn } from 'typeorm';

export enum EscrowStatus {
  ACTIVE = 'active',
  APPROVED = 'approved',
  RELEASED = 'released',
  REFUNDED = 'refunded',
  DISPUTED = 'disputed',
  RESOLVED = 'resolved',
}

@Entity('escrows')
@Index(['escrowId'])
@Index(['depositor'])
@Index(['beneficiary'])
@Index(['status'])
export class Escrow {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'bigint', unique: true })
  escrowId: string;

  @Column()
  depositor: string;

  @Column()
  beneficiary: string;

  @Column({ type: 'bigint' })
  amount: string;

  @Column({ type: 'simple-array' })
  requiredSigners: string[];

  @Column({ type: 'int' })
  requiredApprovals: number;

  @Column({ type: 'simple-array', default: '' })
  approvers: string[];

  @Column({ type: 'int', default: 0 })
  approvalCount: number;

  @Column({
    type: 'enum',
    enum: EscrowStatus,
    default: EscrowStatus.ACTIVE,
  })
  status: EscrowStatus;

  @Column({ type: 'bigint', nullable: true })
  deadline: string;

  @Column({ type: 'text', nullable: true })
  disputeReason: string;

  @Column({ nullable: true })
  disputer: string;

  @Column({ type: 'text', nullable: true })
  resolutionOutcome: string;

  @Column({ type: 'bigint' })
  createdAtLedger: string;

  @Column()
  createdTxHash: string;

  @Column({ type: 'bigint', nullable: true })
  completedAtLedger: string;

  @Column({ nullable: true })
  completedTxHash: string;

  @CreateDateColumn()
  indexedAt: Date;
}
