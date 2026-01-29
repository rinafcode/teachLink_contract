export interface CreditScoreUpdatedEvent {
  user: string;
  new_score: string;
}

export interface CourseCompletedEvent {
  user: string;
  course_id: string;
  points: string;
}

export interface ContributionRecordedEvent {
  user: string;
  c_type: string;
  points: string;
}

export type ScoringEvent =
  | { type: 'CreditScoreUpdatedEvent'; data: CreditScoreUpdatedEvent }
  | { type: 'CourseCompletedEvent'; data: CourseCompletedEvent }
  | { type: 'ContributionRecordedEvent'; data: ContributionRecordedEvent };
