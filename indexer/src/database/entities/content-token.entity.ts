import { Entity, Column, PrimaryGeneratedColumn, Index, CreateDateColumn } from 'typeorm';

@Entity('content_tokens')
@Index(['tokenId'])
@Index(['creator'])
@Index(['currentOwner'])
export class ContentToken {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'bigint', unique: true })
  tokenId: string;

  @Column()
  creator: string;

  @Column()
  currentOwner: string;

  @Column({ type: 'text' })
  contentHash: string;

  @Column({ type: 'text', nullable: true })
  metadataUri: string;

  @Column({ type: 'jsonb', nullable: true })
  metadata: Record<string, any>;

  @Column({ type: 'boolean', default: true })
  transferable: boolean;

  @Column({ type: 'int', default: 0 })
  royaltyPercentage: number;

  @Column({ type: 'int', default: 0 })
  transferCount: number;

  @Column({ type: 'bigint' })
  mintedAtLedger: string;

  @Column()
  mintedTxHash: string;

  @Column({ type: 'bigint' })
  mintedTimestamp: string;

  @Column({ type: 'bigint', nullable: true })
  lastTransferLedger: string;

  @Column({ nullable: true })
  lastTransferTxHash: string;

  @Column({ type: 'bigint', nullable: true })
  lastTransferTimestamp: string;

  @CreateDateColumn()
  indexedAt: Date;
}
