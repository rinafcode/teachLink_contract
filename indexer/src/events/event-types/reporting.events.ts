export interface ReportGeneratedEvent {
  report_id: string;
  report_type: string;
  generated_by: string;
  period_start: string;
  period_end: string;
}

export interface ReportScheduledEvent {
  schedule_id: string;
  template_id: string;
  owner: string;
  next_run_at: string;
}

export interface ReportCommentAddedEvent {
  report_id: string;
  comment_id: string;
  author: string;
}

export interface AlertTriggeredEvent {
  rule_id: string;
  condition_type: string;
  current_value: string;
  threshold: string;
  triggered_at: string;
}

export type ReportingEvent =
  | { type: 'ReportGeneratedEvent'; data: ReportGeneratedEvent }
  | { type: 'ReportScheduledEvent'; data: ReportScheduledEvent }
  | { type: 'ReportCommentAddedEvent'; data: ReportCommentAddedEvent }
  | { type: 'AlertTriggeredEvent'; data: AlertTriggeredEvent };
