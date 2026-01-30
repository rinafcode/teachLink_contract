import { Entity, Column, PrimaryGeneratedColumn, Index, CreateDateColumn } from 'typeorm';

export enum BridgeStatus {
  INITIATED = 'initiated',
  COMPLETED = 'completed',
  FAILED = 'failed',
}

@Entity('bridge_transactions')
@Index(['nonce'])
@Index(['from'])
@Index(['status'])
@Index(['destinationChain'])
export class BridgeTransaction {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'bigint', unique: true })
  nonce: string;

  @Column()
  from: string;

  @Column({ type: 'bigint' })
  amount: string;

  @Column()
  destinationChain: string;

  @Column()
  destinationAddress: string;

  @Column({ nullable: true })
  sourceChain: string;

  @Column({ nullable: true })
  recipient: string;

  @Column({
    type: 'enum',
    enum: BridgeStatus,
    default: BridgeStatus.INITIATED,
  })
  status: BridgeStatus;

  @Column({ type: 'bigint' })
  ledger: string;

  @Column()
  txHash: string;

  @Column({ type: 'bigint' })
  timestamp: string;

  @CreateDateColumn()
  indexedAt: Date;
}
