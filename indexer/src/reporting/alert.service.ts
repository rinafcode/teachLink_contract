import { Injectable, Logger } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { AlertRule, AlertLog } from '@database/entities';
import { AlertConditionType } from '@database/entities/alert-rule.entity';
import { DashboardService } from './dashboard.service';

/**
 * Evaluates alert rules against current metrics and logs triggered alerts
 * for real-time reporting and alerting.
 */
@Injectable()
export class AlertService {
  private readonly logger = new Logger(AlertService.name);

  constructor(
    @InjectRepository(AlertRule)
    private alertRuleRepo: Repository<AlertRule>,
    @InjectRepository(AlertLog)
    private alertLogRepo: Repository<AlertLog>,
    private dashboardService: DashboardService,
  ) {}

  /**
   * Evaluate all enabled alert rules and create log entries for breaches.
   */
  async evaluateAlerts(): Promise<string[]> {
    const analytics = await this.dashboardService.getCurrentAnalytics();
    const rules = await this.alertRuleRepo.find({ where: { enabled: true } });
    const triggered: string[] = [];

    for (const rule of rules) {
      const { currentValue, breached } = this.evaluateRule(rule, analytics);
      if (breached) {
        const log = this.alertLogRepo.create({
          ruleId: rule.ruleId ?? rule.id,
          conditionType: rule.conditionType,
          currentValue: currentValue.toString(),
          threshold: rule.threshold,
          triggeredAt: Math.floor(Date.now() / 1000).toString(),
        });
        await this.alertLogRepo.save(log);
        triggered.push(rule.id);
        this.logger.warn(`Alert triggered: rule=${rule.name} current=${currentValue} threshold=${rule.threshold}`);
      }
    }
    return triggered;
  }

  private evaluateRule(
    rule: AlertRule,
    a: Awaited<ReturnType<DashboardService['getCurrentAnalytics']>>,
  ): { currentValue: number; breached: boolean } {
    let currentValue: number;
    switch (rule.conditionType) {
      case AlertConditionType.BRIDGE_HEALTH_BELOW:
        currentValue = a.bridgeHealthScore;
        return { currentValue, breached: currentValue < Number(rule.threshold) };
      case AlertConditionType.ESCROW_DISPUTE_RATE_ABOVE:
        currentValue =
          a.escrowTotalCount > 0
            ? (a.escrowDisputeCount / a.escrowTotalCount) * 10000
            : 0;
        return { currentValue, breached: currentValue > Number(rule.threshold) };
      case AlertConditionType.VOLUME_ABOVE:
        currentValue = Number(a.bridgeTotalVolume);
        return { currentValue, breached: currentValue > Number(rule.threshold) };
      case AlertConditionType.VOLUME_BELOW:
        currentValue = Number(a.bridgeTotalVolume);
        return { currentValue, breached: currentValue < Number(rule.threshold) };
      case AlertConditionType.TRANSACTION_COUNT_ABOVE:
        currentValue = a.bridgeTotalTransactions;
        return { currentValue, breached: currentValue > Number(rule.threshold) };
      default:
        return { currentValue: 0, breached: false };
    }
  }

  @Cron(CronExpression.EVERY_5_MINUTES)
  async runScheduledEvaluation(): Promise<void> {
    try {
      const triggered = await this.evaluateAlerts();
      if (triggered.length > 0) {
        this.logger.log(`Alerts triggered: ${triggered.join(', ')}`);
      }
    } catch (err) {
      this.logger.error('Scheduled alert evaluation failed', err);
    }
  }

  async getAlertLogs(ruleId?: string, since?: string): Promise<AlertLog[]> {
    const qb = this.alertLogRepo.createQueryBuilder('l').orderBy('l.triggeredAt', 'DESC').take(100);
    if (ruleId) qb.andWhere('l.ruleId = :ruleId', { ruleId });
    if (since) qb.andWhere('l.triggeredAt >= :since', { since });
    return qb.getMany();
  }
}
