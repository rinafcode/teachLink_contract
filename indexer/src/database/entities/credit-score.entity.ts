import { Entity, Column, PrimaryGeneratedColumn, Index, CreateDateColumn, UpdateDateColumn } from 'typeorm';

@Entity('credit_scores')
@Index(['userAddress'])
@Index(['score'])
export class CreditScore {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ unique: true })
  userAddress: string;

  @Column({ type: 'bigint' })
  score: string;

  @Column({ type: 'int', default: 0 })
  coursesCompleted: number;

  @Column({ type: 'int', default: 0 })
  contributionsCount: number;

  @Column({ type: 'bigint' })
  lastUpdatedLedger: string;

  @Column()
  lastUpdatedTxHash: string;

  @Column({ type: 'bigint' })
  lastUpdatedTimestamp: string;

  @CreateDateColumn()
  indexedAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;
}
