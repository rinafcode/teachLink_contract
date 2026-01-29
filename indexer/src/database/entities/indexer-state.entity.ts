import { Entity, Column, PrimaryGeneratedColumn, UpdateDateColumn } from 'typeorm';

@Entity('indexer_state')
export class IndexerState {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ unique: true })
  key: string;

  @Column({ type: 'bigint' })
  lastProcessedLedger: string;

  @Column({ nullable: true })
  lastProcessedTxHash: string;

  @Column({ type: 'bigint', nullable: true })
  lastProcessedTimestamp: string;

  @Column({ type: 'int', default: 0 })
  totalEventsProcessed: number;

  @Column({ type: 'int', default: 0 })
  totalErrors: number;

  @UpdateDateColumn()
  updatedAt: Date;
}
