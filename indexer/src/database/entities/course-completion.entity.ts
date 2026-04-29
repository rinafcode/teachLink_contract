import { Entity, Column, PrimaryGeneratedColumn, Index, CreateDateColumn } from 'typeorm';

@Entity('course_completions')
@Index(['userAddress'])
@Index(['courseId'])
@Index(['completedAt'])
export class CourseCompletion {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column()
  userAddress: string;

  @Column({ type: 'bigint' })
  courseId: string;

  @Column({ type: 'bigint' })
  pointsEarned: string;

  @Column({ type: 'bigint' })
  completedAt: string;

  @Column({ type: 'bigint' })
  ledger: string;

  @Column()
  txHash: string;

  @CreateDateColumn()
  indexedAt: Date;
}
