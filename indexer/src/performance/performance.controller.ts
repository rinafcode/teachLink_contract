import { Controller, Get } from '@nestjs/common';
import { MetricsService } from './metrics.service';

/**
 * Performance monitoring and load-balancer health.
 * - GET /health: liveness/readiness for load balancers.
 * - GET /metrics: JSON snapshot for performance monitoring and alerting.
 */
@Controller()
export class PerformanceController {
  constructor(private readonly metricsService: MetricsService) {}

  @Get('health')
  getHealth(): { status: string; timestamp: string } {
    this.metricsService.recordRequest();
    return {
      status: 'ok',
      timestamp: new Date().toISOString(),
    };
  }

  @Get('metrics')
  getMetrics() {
    this.metricsService.recordRequest();
    return this.metricsService.getSnapshot();
  }
}
