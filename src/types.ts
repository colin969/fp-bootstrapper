export enum OperatingSystem {
  LINUX = "LINUX",
  WINDOWS = "WINDOWS",
  MACOS = "MACOS"
}

export enum View {
  SETUP = "SETUP",
  SETUPSELECT = "SETUPSELECT",
  INSTALLATION = "INSTALLATION",
}

export type OsConfig = {
  default_path: string;
  relative_executable: string;
}

export type AppConfig = {
  name: string;
  windows?: OsConfig;
  linux?: OsConfig;
  macos?: OsConfig;
}

export type AppState = {
  fatal_error?: string;
  operating_system: OperatingSystem;
  view: View;
  installation_target: OperatingSystem;
  installation_path: string;
  installation_channel: string;
  components: ComponentList;
  config: AppConfig;
}

export function osToName(os: OperatingSystem) {
  switch(os) {
    case OperatingSystem.LINUX: {
      return "Linux";
    }
    case OperatingSystem.MACOS: {
      return "MacOS";
    }
    case OperatingSystem.WINDOWS: {
      return "Windows";
    }
  }
}

export type ComponentList = {
  url: string;
  categories: Category[];
  selected: string[];
  required: string[];
}

export type Category = {
  id: string;
  name: string;
  description: string;
  subcategories: Category[]; // Optional to account for nested categories
  components: Component[];
}

export type Component = {
  id: string;
  name: string;
  description: string;
  date_modified: string;
  download_size: number;
  install_size: number;
  path?: string;
  hash: string;
  depends?: string;
  required: boolean;
  installed: boolean;
}