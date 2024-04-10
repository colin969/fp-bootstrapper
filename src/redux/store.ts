import { configureStore } from '@reduxjs/toolkit';
import stateReducer from './state/stateSlice';

export const store = configureStore({
  reducer: {
    state: stateReducer,
  },
});

// Infer the `RootState` and `AppDispatch` types from the store itself
export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;