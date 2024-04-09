import { Box, Button, FormControl, FormControlLabel, FormLabel, Radio, RadioGroup, TextField } from "@mui/material";
import { useCallback, useContext, useState } from "react";
import { StateContext } from "../StateContext";
import { View, osToName } from "../types";
import { message, open } from '@tauri-apps/api/dialog';
import { invoke } from "@tauri-apps/api";

export type SetupPageProps = {
  setBusy: (busy: boolean) => void;
}

export function SetupPage(props: SetupPageProps) {
  const { state } = useContext(StateContext);
  const [installPath, setInstallPath] = useState(state.installation_path);

  const showSelectDialog = useCallback(() => {
    open({
      multiple: false,
      directory: true,
      defaultPath: state.installation_path,
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
      <h2>Operating System: {osToName(state.operating_system)}</h2>
      <FormControl fullWidth>
        <FormLabel id="radio-buttons-group-label">Version to Install</FormLabel>
        <RadioGroup
          aria-labelledby="radio-buttons-group-label"
          value={state.installation_target}
          name="radio-buttons-group">
          { state.config.windows && <FormControlLabel value="WINDOWS" control={<Radio />} label="Windows" /> }
          { state.config.linux && <FormControlLabel value="LINUX" control={<Radio />} label="Linux" /> }
          { state.config.macos && <FormControlLabel value="MACOS" control={<Radio />} label="MacOS" /> }
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
            props.setBusy(true);
            invoke('change_view', { view: View.SETUPSELECT })
            .catch((error) => {
              message(error, 'Error');
            })
            .finally(() => {
              props.setBusy(false);
            });
          }}>Next</Button>
        </Box>
      </FormControl>
    </>
  );
}