import { Controller, Get, Header, Res } from '@nestjs/common';
import { Response } from 'express';
import { HealthService } from './health.service';
import { MetricsService } from './metrics.service';

/**
 * Performance monitoring and load-balancer health.
 * - GET /health: liveness/readiness with dependency checks.
 * - GET /metrics: Prometheus metrics for monitoring and alerting.
 * - GET /metrics/json: basic JSON metadata for quick debugging.
 */
@Controller()
export class PerformanceController {
  constructor(
    private readonly metricsService: MetricsService,
    private readonly healthService: HealthService,
  ) {}

  @Get('health')
  getHealth() {
    return this.healthService.getHealthStatus();
  }

  @Get('metrics')
  @Header('Content-Type', 'text/plain; version=0.0.4; charset=utf-8')
  async getMetrics(@Res({ passthrough: true }) response: Response): Promise<string> {
    response.setHeader('Content-Type', this.metricsService.getContentType());
    return this.metricsService.getPrometheusMetrics();
  }

  @Get('metrics/json')
  async getMetricsJson() {
    const health = await this.healthService.getHealthStatus();
    return {
      status: health.status,
      service: health.service,
      timestamp: health.timestamp,
      details: health.details,
    };
  }
}
