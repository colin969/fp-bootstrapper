import { Box, Button, FormControl, FormControlLabel, FormLabel, Radio, RadioGroup, TextField } from "@mui/material";
import { invoke } from "@tauri-apps/api";
import { message, open } from '@tauri-apps/api/dialog';
import { useCallback, useState } from "react";
import { useDispatch, useSelector } from "react-redux";
import { RootState } from "../redux/store";
import { View, osToName } from "../types";
import { setBusy } from "../redux/state/stateSlice";

export function SetupPage() {
  const { appState } = useSelector((state: RootState) => state.state);
  const [installPath, setInstallPath] = useState(appState.installation_path);
  const dispatch = useDispatch();

  const showSelectDialog = useCallback(() => {
    open({
      multiple: false,
      directory: true,
      defaultPath: appState.installation_path,
    })
    .then((chosenPath) => {
      if (typeof chosenPath === 'string') {
        setInstallPath(chosenPath);
        invoke('set_installation_path', { path: chosenPath });
      }
    }).catch(() => {});
  }, []);

  return (
    <>
      <h1 className="title">Setup</h1>
      <h2>Operating System: {osToName(appState.operating_system)}</h2>
      <FormControl fullWidth>
        <FormLabel id="radio-buttons-group-label">Version to Install</FormLabel>
        <RadioGroup
          aria-labelledby="radio-buttons-group-label"
          value={appState.installation_target}
          name="radio-buttons-group">
          { appState.config.windows && <FormControlLabel value="WINDOWS" control={<Radio />} label="Windows" /> }
          { appState.config.linux && <FormControlLabel value="LINUX" control={<Radio />} label="Linux" /> }
          { appState.config.macos && <FormControlLabel value="MACOS" control={<Radio />} label="MacOS" /> }
        </RadioGroup>
        <Box className='box-row' sx={{ display: 'flex', alignItems: 'center' }}>
          <TextField fullWidth value={installPath} onChange={(event) => {
            setInstallPath(event.currentTarget.value);
            invoke('set_installation_path', { path: event.currentTarget.value });
          }}/>
          <Button 
            variant='contained'
            onClick={showSelectDialog}>
            Choose Installation Path
          </Button>
        </Box>
        <Box sx={{ marginTop: '20px' }}>
          <Button variant='contained' onClick={() => {
            dispatch(setBusy(true));
            invoke('change_view', { view: View.SETUPSELECT })
            .catch((error) => {
              message(error, 'Error');
            })
            .finally(() => {
              dispatch(setBusy(false));
            });
          }}>Next</Button>
        </Box>
      </FormControl>
    </>
  );
}