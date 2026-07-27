/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@/api/invokeRuntime'
import type { ConverterRequestDto } from '@/types/generated/converter/ConverterRequestDto'
import type { ConvertResult } from '@/types/generated/converter/ConvertResult'

export const convertConfig = (request: ConverterRequestDto): Promise<ConvertResult> =>
  invoke('convert_config', { request })
