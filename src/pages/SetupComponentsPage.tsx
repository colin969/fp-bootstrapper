import { Box } from "@mui/material";
import { useSelector } from "react-redux";
import { ComponentsTreeView } from "../components/ComponentTreeView";
import { RootState } from "../redux/store";

export function SetupComponentsPage() {
  const appState = useSelector((state: RootState) => state.state);

  return (
    <>
      <h1 className='title'>Installation Options</h1>
      <Box>
        <ComponentsTreeView />
      </Box>
    </>
  )
}