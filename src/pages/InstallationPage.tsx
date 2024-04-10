import { useSelector } from "react-redux";
import { RootState } from "../redux/store";
import { Box, Button } from "@mui/material";
import { readableByteSize } from "../util";

export function InstallationPage() {
  const { downloadState } = useSelector((state: RootState) => state.state);

  return (
    <div>
      <h1>Installation</h1>
      <Box>
        <h3>{`${(downloadState.total_components + 1) - downloadState.component_number} Remaining...`}</h3>
        <h2>{`Installing: ${downloadState.current?.name}`}</h2>
        <h3>{downloadState.stage === "Downloading" ? `Download Rate: ${readableByteSize(downloadState.download_rate)}` : '...'}</h3>
        <h3>{`${downloadState.stage}...`}</h3>
      </Box>
      <Box className='box-row'>
        <Button variant="contained">
          Abort
        </Button>
      </Box>
    </div>
  )
}