/* Generated from commands/handler_registry.rs; do not edit. */

import { invoke } from '@/api/invokeRuntime'
import type { BenchmarkS2SnapshotDto } from '@/types/generated/benchmark/BenchmarkS2SnapshotDto'
import type { BenchmarkS2CancelRequest } from '@/types/generated/benchmark/BenchmarkS2CancelRequest'

export const startBenchmarkS2 = (): Promise<BenchmarkS2SnapshotDto> => invoke('benchmark_s2_start')
export const getBenchmarkS2Status = (): Promise<BenchmarkS2SnapshotDto> => invoke('benchmark_s2_get_status')
export const cancelBenchmarkS2 = (request: BenchmarkS2CancelRequest): Promise<BenchmarkS2SnapshotDto> => invoke('benchmark_s2_cancel', { request })
