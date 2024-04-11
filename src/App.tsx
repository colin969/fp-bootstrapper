import { Backdrop } from '@mui/material';
import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from "react";
import { useDispatch, useSelector } from 'react-redux';
import "./App.css";
import { InstallationPage } from './pages/InstallationPage';
import { SetupComponentsPage } from './pages/SetupComponentsPage';
import { SetupPage } from './pages/SetupPage';
import { DownloadState, setDownloadState, setSelected, setState } from './redux/state/stateSlice';
import { RootState } from './redux/store';
import { AppState, View } from './types';
import { FailurePage } from './pages/FailurePage';

function App() {
  const { appState, busy } = useSelector((state: RootState) => state.state);
  const dispatch = useDispatch();
  const [fatalError, setFatalError] = useState<string>("");

  useEffect(() => {
    // Listen for any state changes
    listen<AppState>('sync', (event) => {
      dispatch(setState(event.payload));
    });

    listen<string[]>('sync_selected', (event) => {
      dispatch(setSelected(event.payload));
    })

    listen<DownloadState>('download_state', (event) => {
      dispatch(setDownloadState(event.payload));
    });

    listen<number>('installation_finished', (_) => {
      invoke<AppState>('installation_finished_back')
      .then((newState) => {
        dispatch(setState(newState));
      })
      .catch(setFatalError);
    });

    // Listen for any fatal errors
    listen<string>('fatal_error', (event) => {
      setFatalError(event.payload);
    })
    // Tell the backend we're ready, and get the initial state back
    invoke<AppState>('init_process')
    .then((newState) => {
      dispatch(setState(newState));
    })
    .catch(setFatalError);
  }, []);

  const renderView = (): JSX.Element => {
    switch(appState.view) {
      case View.SETUP: {
        return <SetupPage/>;
      }
      case View.SETUPSELECT: {
        return <SetupComponentsPage/>
      }
      case View.INSTALLATION: {
        return <InstallationPage />
      }
      default: {
        return <>
          {appState.view}
        </>;
      }
    }
  };

  return (
    <>
    <div className="content">
      {fatalError ? (
        <FailurePage failure={fatalError}/>
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
      )}
    </div><Backdrop open={busy} />
    </>
  );
}

export default App;
