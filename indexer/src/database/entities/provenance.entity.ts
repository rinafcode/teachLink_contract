import { Entity, Column, PrimaryGeneratedColumn, Index, CreateDateColumn } from 'typeorm';

export enum ProvenanceEventType {
  MINT = 'mint',
  TRANSFER = 'transfer',
  METADATA_UPDATE = 'metadata_update',
}

@Entity('provenance_records')
@Index(['tokenId'])
@Index(['fromAddress'])
@Index(['toAddress'])
@Index(['eventType'])
@Index(['timestamp'])
export class ProvenanceRecord {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'bigint' })
  tokenId: string;

  @Column({
    type: 'enum',
    enum: ProvenanceEventType,
  })
  eventType: ProvenanceEventType;

  @Column({ nullable: true })
  fromAddress: string;

  @Column()
  toAddress: string;

  @Column({ type: 'bigint' })
  timestamp: string;

  @Column({ type: 'bigint' })
  ledger: string;

  @Column()
  txHash: string;

  @Column({ type: 'jsonb', nullable: true })
  additionalData: Record<string, any>;

  @CreateDateColumn()
  indexedAt: Date;
}
