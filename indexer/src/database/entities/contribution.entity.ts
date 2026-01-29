import { Entity, Column, PrimaryGeneratedColumn, Index, CreateDateColumn } from 'typeorm';

@Entity('contributions')
@Index(['userAddress'])
@Index(['contributionType'])
@Index(['timestamp'])
export class Contribution {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column()
  userAddress: string;

  @Column()
  contributionType: string;

  @Column({ type: 'bigint' })
  pointsEarned: string;

  @Column({ type: 'text', nullable: true })
  description: string;

  @Column({ type: 'bigint' })
  timestamp: string;

  @Column({ type: 'bigint' })
  ledger: string;

  @Column()
  txHash: string;

  @CreateDateColumn()
  indexedAt: Date;
}
