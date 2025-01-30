export interface BotItem {
  id: string;
  flag: string;
  name: string;
  cpuBrand: string;
  ram: string;
  ping: string;
  joinDate: string;
  systemBootTime: number;
  region: string;
  osInfo: string;
  activeWindow: string;
  mibConfig: MibCnfg | null;
  clientInstances: FrontendClientInstance[];
  loaderInstances: FrontendLoaderInstance[];
}

export interface FrontendClientInstance {
  joinDate: string;
  version: number;
}

export interface FrontendLoaderInstance {
  joinDate: string;
  version: number;
  tag: string;
}

export interface HstCnfg {
  version: number;
  folderPath: string;
  exePath: string;
  configPath: string;
}

export interface MibCnfg {
  cleanupDone: boolean;
  loaders: HstCnfg[];
  clients: HstCnfg[];
}
