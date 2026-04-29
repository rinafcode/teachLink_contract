import { LoggerService as NestLoggerService, Injectable, Scope } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';

@Injectable({ scope: Scope.TRANSIENT })
export class LoggerService implements NestLoggerService {
  private context?: string;
  private readonly isProduction: boolean;
  private readonly logLevel: string;

  constructor(private configService: ConfigService) {
    this.isProduction = this.configService.get<string>('app.nodeEnv') === 'production';
    this.logLevel = this.configService.get<string>('app.logLevel') || 'info';
  }

  setContext(context: string) {
    this.context = context;
  }

  private shouldLog(level: string): boolean {
    const levels = ['debug', 'info', 'warn', 'error'];
    const configuredIndex = levels.indexOf(this.logLevel.toLowerCase());
    const targetIndex = levels.indexOf(level);
    return targetIndex >= configuredIndex;
  }

  private formatMessage(level: string, message: any, optionalParams: any[]) {
    const timestamp = new Date().toISOString();
    
    let metadata = {};
    let currentContext = this.context || 'Application';

    if (optionalParams.length > 0) {
      // Extract trailing string arguments (usually used as the context internally by NestJS)
      if (typeof optionalParams[optionalParams.length - 1] === 'string') {
        currentContext = optionalParams.pop();
      }
      
      // Next trailing param logic checks if it's an object intended for structured metadata logging
      if (optionalParams.length > 0) {
        const lastParam = optionalParams[optionalParams.length - 1];
        if (typeof lastParam === 'object' && lastParam !== null) {
          metadata = optionalParams.pop();
        }
      }
    }

    const logEntry = {
      timestamp,
      level,
      context: currentContext,
      message: typeof message === 'object' ? JSON.stringify(message) : message,
      ...(Object.keys(metadata).length > 0 ? { data: metadata } : {}),
    };

    if (this.isProduction) {
      return JSON.stringify(logEntry);
    }

    // Human-readable format output tailored for development environments
    const colorCode = this.getColorCode(level);
    const resetCode = '\x1b[0m';
    const metadataStr = Object.keys(metadata).length > 0 ? `\n\x1b[33m[Data]: ${JSON.stringify(metadata, null, 2)}\x1b[0m` : '';
    
    return `${colorCode}[${timestamp}] [${level.toUpperCase()}] [${currentContext}] ${logEntry.message}${resetCode}${metadataStr}`;
  }

  private getColorCode(level: string): string {
    switch (level) {
      case 'error': return '\x1b[31m'; // Red
      case 'warn': return '\x1b[33m';  // Yellow
      case 'info': return '\x1b[32m';  // Green
      case 'debug': return '\x1b[36m'; // Cyan
      default: return '\x1b[37m';      // White
    }
  }

  log(message: any, ...optionalParams: any[]) {
    if (!this.shouldLog('info')) return;
    console.log(this.formatMessage('info', message, optionalParams));
  }

  error(message: any, ...optionalParams: any[]) {
    if (!this.shouldLog('error')) return;
    console.error(this.formatMessage('error', message, optionalParams));
  }

  warn(message: any, ...optionalParams: any[]) {
    if (!this.shouldLog('warn')) return;
    console.warn(this.formatMessage('warn', message, optionalParams));
  }

  debug(message: any, ...optionalParams: any[]) {
    if (!this.shouldLog('debug')) return;
    console.debug(this.formatMessage('debug', message, optionalParams));
  }

  verbose?(message: any, ...optionalParams: any[]) {
    if (!this.shouldLog('debug')) return;
    console.debug(this.formatMessage('debug', message, optionalParams));
  }
}