import { NestFactory } from '@nestjs/core';
import { Logger } from 'nestjs-pino';
import { ConfigService } from '@nestjs/config';
import { AppModule } from './app.module';

async function bootstrap() {
  const app = await NestFactory.create(AppModule, {
    bufferLogs: true,
  });

  const configService = app.get(ConfigService);
  // Configure pino logger based on settings
  const isPretty = configService.get('logging.format') === 'pretty';
  const loggerOptions = {
    level: configService.get('logging.level'),
    transport: isPretty 
      ? {
          target: 'pino-pretty',
          options: {
            colorize: true,
            translateTime: 'SYS:ss:yyyy-mm-dd HH:MM:ss',
            ignore: 'pid,hostname'
          }
        }
      : undefined
  };
  
  // Use Pino logger with configuration
  app.useLogger(app.get(Logger, loggerOptions));

  const port = process.env.PORT || 3000;
  await app.listen(port);

  const logger = app.get(Logger);
  logger.log(`TeachLink Indexer is running on port ${port}`);
  logger.log(`Environment: ${process.env.NODE_ENV || 'development'}`);
  logger.log(`Network: ${process.env.STELLAR_NETWORK || 'testnet'}`);
  
  // Log configuration info
  logger.log(`Log level: ${configService.get('logging.level')}`);
  logger.log(`Logging format: ${configService.get('logging.format')}`);
}

bootstrap();
