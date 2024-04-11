import { PayloadAction, createSlice } from '@reduxjs/toolkit';
import { AppState, Component, OperatingSystem, View } from '../../types';

const initialState: AppState = {
  operating_system: OperatingSystem.LINUX,
  installation_target: OperatingSystem.LINUX,
  installation_path: './Flashpoint',
  installation_channel: 'Stable',
  components: { url: '', categories: [], selected: [], required: [] },
  view: View.SETUP,
  config: {
    name: 'Example App',
  },
}

export type DownloadState = {
  total_size: number,
  total_downloaded: number,
  total_components: number,
  component_number: number,
  current?: Component,
  stage: String,
}

export type GlobalState = {
  appState: AppState,
  busy: boolean,
  downloadState: DownloadState,
};

export const stateSlice = createSlice({
  name: 'state',
  initialState: {
    appState: initialState,
    busy: false,
    downloadState: {
      download_rate: 0,
      total_size: 0,
      total_downloaded: 0,
      component_number: 0,
      total_components: 0,
      current: undefined,
      stage: 'Downloading',
    }
  } as GlobalState,
  reducers: {
    setState: (state, action: PayloadAction<AppState>) => {
      Object.assign(state.appState, action.payload);
    },
    setSelected: (state, action: PayloadAction<string[]>) => {
      state.appState.components.selected = action.payload;
    },
    setBusy: (state, action: PayloadAction<boolean>) => {
      state.busy = action.payload;
    },
    setDownloadState: (state, action: PayloadAction<DownloadState>) => {
      state.downloadState = action.payload;
    },
  },
});

export const { setState, setSelected, setBusy, setDownloadState } = stateSlice.actions;

export default stateSlice.reducer;