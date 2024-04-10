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
  initialState,
  reducers: {
    setState: (state, action: PayloadAction<AppState>) => {
      Object.assign(state, action.payload);
    },
    setSelected: (state, action: PayloadAction<string[]>) => {
      state.components.selected = action.payload;
    },
  },
});

export const { setState, setSelected } = stateSlice.actions;

export default stateSlice.reducer;