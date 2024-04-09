import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from "react";
import "./App.css";
import { DEFAULT_STATE, StateContext } from './StateContext';
import { SetupPage } from './pages/SetupPage';
import { AppState, View } from './types';
import { SetupComponentsPage } from './pages/SetupComponentsPage';
import { Backdrop } from '@mui/material';

function App() {
  const [appState, setAppState] = useState<AppState>(DEFAULT_STATE);
  const [fatalError, setFatalError] = useState<string>("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    // Listen for any state changes
    listen<AppState>('sync', (event) => {
      setAppState(event.payload);
    });

    // Listen for any fatal errors
    listen<string>('fatal_error', (event) => {
      setFatalError(event.payload);
    })

    // Tell the backend we're ready, and get the initial state back
    invoke<AppState>('init_process')
    .then(setAppState)
    .catch(setFatalError);
    
  }, []);

  const renderView = (): JSX.Element => {
    switch(appState.view) {
      case View.SETUP: {
        return <SetupPage setBusy={setBusy}/>;
      }
      case View.SETUPSELECT: {
        return <SetupComponentsPage/>
      }
      default: {
        return <>
          {appState.view}
        </>;
      }
    }
  };

  return (
    <StateContext.Provider value={{
      state: appState,
      setState: setAppState,
    }}>
      <div className="content">
        { fatalError ? (
          <h1>FATAL ERROR: {fatalError}</h1>
        ) : (appState && !appState.fatal_error) ? (
          <>
            {renderView()}
          </>
        ) : (appState.fatal_error) ? (
          <>
            <h1>Fatal Error</h1>
            <h2>{appState.fatal_error}</h2>
          </>
        ) : (
          <h1>How did you get here?</h1>
        ) }
      </div>
      <Backdrop open={busy} />
    </StateContext.Provider>
  );
}

export default App;
