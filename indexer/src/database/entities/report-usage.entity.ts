import {
  Entity,
  Column,
  PrimaryGeneratedColumn,
  Index,
  CreateDateColumn,
} from 'typeorm';

/**
 * Tracks report view/usage for analytics.
 * Can be populated from chain events or API view calls.
 */
@Entity('report_usage')
@Index(['reportId'])
@Index(['viewer'])
@Index(['viewedAt'])
export class ReportUsage {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'bigint' })
  reportId: string;

  @Column()
  viewer: string;

  @Column({ type: 'bigint' })
  viewedAt: string;

  @CreateDateColumn()
  indexedAt: Date;
}
