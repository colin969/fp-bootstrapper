import { createContext } from 'react';
import { AppState, FrontState, OperatingSystem, View } from './types';

export const DEFAULT_STATE: AppState = {
  operating_system: OperatingSystem.LINUX,
  installation_target: OperatingSystem.LINUX,
  installation_path: './Flashpoint',
  installation_channel: 'Stable',
  components: { url: '', categories: [] },
  view: View.SETUP,
  config: {
    name: 'Example App',
  },
};
export const StateContext = createContext<FrontState>({
  state: DEFAULT_STATE,
  setState: () => {}
});