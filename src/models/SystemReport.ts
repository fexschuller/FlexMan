export interface SystemReport {
  flexasioVersion: string | null;
  flexasioInstallDir: string | null;
  configPath: string;
  configExists: boolean;
  logPath: string;
  logExists: boolean;
  osVersion: string;
}
