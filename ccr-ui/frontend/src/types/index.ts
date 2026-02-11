// TypeScript type definitions for CCR UI
//
// Domain types are organized into separate modules.
// This barrel file re-exports everything for backward compatibility.

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  message?: string;
}

// Domain type re-exports
export * from './stats'
export * from './config'
export * from './mcp'
export * from './codex'
export * from './converter'
export * from './gemini'
export * from './qwen'
export * from './iflow'
export * from './droid'
export * from './sync'
export * from './claude'

// Previously split modules
export * from './checkin'
export * from './api'
export * from './router'
