// WebDAV sync type definitions

export interface SyncConfigDetails {
  enabled: boolean;
  webdav_url: string;
  username: string;
  remote_path: string;
  auto_sync: boolean;
  remote_file_exists?: boolean | null;
}

export interface SyncStatusResponse {
  success: boolean;
  output: string;
  configured: boolean;
  config?: SyncConfigDetails | null;
}

export interface SyncOperationRequest {
  force?: boolean;
}

export interface SyncOperationResponse {
  success: boolean;
  output: string;
  error: string;
  duration_ms: number;
}

export interface SyncInfoResponse {
  feature_name: string;
  description: string;
  supported_services: string[];
  setup_steps: string[];
  security_notes: string[];
}
