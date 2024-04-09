import { useContext } from "react";
import { StateContext } from "../StateContext";
import { ComponentsTreeView } from "../components/ComponentTreeView";
import { Box } from "@mui/material";

export function SetupComponentsPage() {
  const { state } = useContext(StateContext);

  return (
    <>
      <h1 className='title'>Installation Options</h1>
      <Box>
        <ComponentsTreeView list={state.components}/>
      </Box>
    </>
  )
}