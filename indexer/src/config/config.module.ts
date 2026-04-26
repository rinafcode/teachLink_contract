import { Module } from '@nestjs/common';
import { ConfigModule as NestConfigModule } from '@nestjs/config';
import configuration from './configuration';
import { ConfigManager } from './config.manager';

/**
 * Provides centralized, validated configuration to the rest of the application.
 *
 * Import this module once in AppModule. Inject `ConfigManager` wherever you
 * need typed config access or want to trigger a hot-reload.
 */
@Module({
  imports: [
    NestConfigModule.forRoot({
      isGlobal: true,
      load: [configuration],
    }),
  ],
  providers: [ConfigManager],
  exports: [ConfigManager],
})
export class AppConfigModule {}
