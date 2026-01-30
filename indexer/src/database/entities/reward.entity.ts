import { Entity, Column, PrimaryGeneratedColumn, Index, CreateDateColumn } from 'typeorm';

export enum RewardStatus {
  ISSUED = 'issued',
  CLAIMED = 'claimed',
}

@Entity('rewards')
@Index(['recipient'])
@Index(['rewardType'])
@Index(['status'])
@Index(['timestamp'])
export class Reward {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column()
  recipient: string;

  @Column({ type: 'bigint' })
  amount: string;

  @Column()
  rewardType: string;

  @Column({
    type: 'enum',
    enum: RewardStatus,
    default: RewardStatus.ISSUED,
  })
  status: RewardStatus;

  @Column({ type: 'bigint' })
  timestamp: string;

  @Column({ type: 'bigint', nullable: true })
  claimedAt: string;

  @Column({ type: 'bigint' })
  ledger: string;

  @Column()
  txHash: string;

  @CreateDateColumn()
  indexedAt: Date;
}
