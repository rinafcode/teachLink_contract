import { Module } from '@nestjs/common';
import { HorizonService } from './horizon.service';

@Module({
  providers: [HorizonService],
  exports: [HorizonService],
})
export class HorizonModule {}
