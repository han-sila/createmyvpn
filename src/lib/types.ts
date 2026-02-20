export interface AwsCredentials {
  access_key_id: string;
  secret_access_key: string;
}

export interface DoCredentials {
  api_token: string;
}

export interface AwsRegion {
  code: string;
  name: string;
}

export type DeploymentStatus =
  | "not_deployed"
  | "deploying"
  | "deployed"
  | "destroying"
  | "failed";

export type VpnConnectionStatus =
  | "disconnected"
  | "connecting"
  | "connected"
  | "disconnecting";

export interface DeploymentState {
  status: DeploymentStatus;
  deployment_mode: string | null; // "aws" | "byo" | "do" (null = "aws" for old state files)
  region: string | null;
  vpc_id: string | null;
  igw_id: string | null;
  subnet_id: string | null;
  route_table_id: string | null;
  security_group_id: string | null;
  key_pair_name: string | null;
  instance_id: string | null;
  allocation_id: string | null;
  association_id: string | null;
  elastic_ip: string | null;
  ssh_private_key: string | null;
  ssh_user: string | null;
  server_public_key: string | null;
  client_private_key: string | null;
  client_public_key: string | null;
  client_config: string | null;
  deployed_at: string | null;
  auto_destroy_at: string | null; // ISO datetime, null if not set
  error_message: string | null;
  // DigitalOcean-specific fields
  droplet_id: number | null;
  do_firewall_id: string | null;
  do_ssh_key_id: number | null;
}

export interface AppSettings {
  region: string;
  instance_type: string;
  wireguard_port: number;
}

export interface ProgressEvent {
  step: number;
  total_steps: number;
  message: string;
  status: "running" | "done" | "error";
}

export interface DoRegion {
  code: string;
  name: string;
}

export const DO_REGIONS: DoRegion[] = [
  { code: "nyc1", name: "New York 1" },
  { code: "nyc3", name: "New York 3" },
  { code: "sfo3", name: "San Francisco 3" },
  { code: "ams3", name: "Amsterdam 3" },
  { code: "lon1", name: "London 1" },
  { code: "fra1", name: "Frankfurt 1" },
  { code: "sgp1", name: "Singapore 1" },
  { code: "blr1", name: "Bangalore 1" },
  { code: "tor1", name: "Toronto 1" },
  { code: "syd1", name: "Sydney 1" },
];

export const AWS_REGIONS: AwsRegion[] = [
  { code: "us-east-1", name: "US East (N. Virginia)" },
  { code: "us-east-2", name: "US East (Ohio)" },
  { code: "us-west-1", name: "US West (N. California)" },
  { code: "us-west-2", name: "US West (Oregon)" },
  { code: "eu-west-1", name: "Europe (Ireland)" },
  { code: "eu-west-2", name: "Europe (London)" },
  { code: "eu-central-1", name: "Europe (Frankfurt)" },
  { code: "eu-north-1", name: "Europe (Stockholm)" },
  { code: "ap-southeast-1", name: "Asia Pacific (Singapore)" },
  { code: "ap-southeast-2", name: "Asia Pacific (Sydney)" },
  { code: "ap-northeast-1", name: "Asia Pacific (Tokyo)" },
  { code: "ap-south-1", name: "Asia Pacific (Mumbai)" },
  { code: "sa-east-1", name: "South America (São Paulo)" },
  { code: "ca-central-1", name: "Canada (Central)" },
  { code: "me-south-1", name: "Middle East (Bahrain)" },
  { code: "af-south-1", name: "Africa (Cape Town)" },
];
