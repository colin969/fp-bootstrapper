import { Box, Button } from "@mui/material";
import { ComponentsTreeView } from "../components/ComponentTreeView";
import { useDispatch } from "react-redux";
import { invoke } from "@tauri-apps/api";
import { setBusy } from "../redux/state/stateSlice";
import { View } from "../types";
import { message } from "@tauri-apps/api/dialog";

export function SetupComponentsPage() {
  const dispatch = useDispatch();
  
  return (
    <div className='vertical-box'>
      <h1 className='title'>Installation Options</h1>
      <Box className='scroll-box'>
        <ComponentsTreeView />
      </Box>
      <div className='filler'/>
      <Box className='box-row'>
        <Button variant='contained' onClick={() => {
          dispatch(setBusy(true));
          invoke('change_view', { view: View.SETUP })
          .catch((error) => {
            message(error, 'Error');
          })
          .finally(() => {
            dispatch(setBusy(false));
          });
        }}>Setup</Button>
        <Button variant='contained' onClick={() => {
          dispatch(setBusy(true));
          invoke('change_view', { view: View.INSTALLATION })
          .catch((error) => {
            message(error, 'Error');
          })
          .finally(() => {
            dispatch(setBusy(false));
          });
        }}>Install</Button>
      </Box>
    </div>
  )
}