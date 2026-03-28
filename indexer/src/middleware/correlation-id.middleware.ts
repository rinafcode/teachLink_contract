import { Injectable, NestMiddleware } from '@nestjs/common';
import { Request, Response, NextFunction } from 'express';

@Injectable()
export class CorrelationIdMiddleware implements NestMiddleware {
  use(req: Request, res: Response, next: NextFunction) {
    const correlationId = req.headers['x-correlation-id'] || 
                         req.get('X-Correlation-ID') ||
                         this.generateId();
    
    req['correlationId'] = correlationId;
    res.setHeader('X-Correlation-ID', correlationId);
    
    next();
  }
  
  private generateId(): string {
    return Math.random().toString(36).substring(2, 15) +
           Math.random().toString(36).substring(2, 15);
  }
}