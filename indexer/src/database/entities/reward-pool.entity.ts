import { Entity, Column, PrimaryGeneratedColumn, CreateDateColumn, UpdateDateColumn } from 'typeorm';

@Entity('reward_pool')
export class RewardPool {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'bigint' })
  totalPoolBalance: string;

  @Column({ type: 'bigint' })
  totalRewardsIssued: string;

  @Column({ type: 'bigint' })
  totalRewardsClaimed: string;

  @Column({ type: 'bigint' })
  lastFundedLedger: string;

  @Column()
  lastFundedTxHash: string;

  @Column({ type: 'bigint' })
  lastFundedTimestamp: string;

  @Column()
  lastFunder: string;

  @Column({ type: 'bigint' })
  lastFundedAmount: string;

  @CreateDateColumn()
  indexedAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;
}
