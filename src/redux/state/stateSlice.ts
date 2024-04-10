import { PayloadAction, createSlice } from '@reduxjs/toolkit';
import { AppState, OperatingSystem, View } from '../../types';

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

export const stateSlice = createSlice({
  name: 'state',
  initialState: {
    appState: initialState,
    busy: false,
  },
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
  },
});

export const { setState, setSelected, setBusy } = stateSlice.actions;

export default stateSlice.reducer;