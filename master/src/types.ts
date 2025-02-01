export interface BotItem {
  id: string;
  flag: string;
  name: string;
  cpu_brand: string;
  ram: string;
  ping: string;
  join_date: string;
  system_boot_time: number;
  region: string;
  os_info: string;
  active_window: string;
  client_instances: FrontendClientInstance[];
  loader_instances: FrontendLoaderInstance[];
}

export interface FrontendClientInstance {
  join_date: string;
  version: number;
}

export interface FrontendLoaderInstance {
  join_date: string;
  version: number;
  tag: string;
}
export interface HstCnfg {
  version: number;
  folder_path: string;
  exe_path: string;
  config_path: string;
}

export interface MibCnfg {
  cln_up_done: boolean;
  ldrs: HstCnfg[];
  clnts: HstCnfg[];
}
