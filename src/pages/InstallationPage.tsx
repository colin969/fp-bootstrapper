import { useSelector } from "react-redux";
import { RootState } from "../redux/store";
import { Box, Button, LinearProgress } from "@mui/material";
import { readableByteSize } from "../util";
import { useMemo } from "react";

export function InstallationPage() {
  const { downloadState } = useSelector((state: RootState) => state.state);
  const percent = (downloadState.total_downloaded / downloadState.total_size) * 100;

  const startTime = useMemo(() => Date.now(), []);
  const elapsedMs = Date.now() - startTime;
  const downloadRate = downloadState.total_downloaded / (elapsedMs / 1000);

  return (
    <div>
      <h1 className="title">Installation</h1>
      <Box>
        <h3>{percent.toFixed(1)}% Progress {`(${readableByteSize(downloadState.total_downloaded)} - ${readableByteSize(downloadState.total_size)})`}</h3>
        <LinearProgress variant="determinate" value={percent}/>
        <h3>{`${(downloadState.total_components + 1) - downloadState.component_number} Components Remaining...`}</h3>
        <h2>{`Installing: ${downloadState.current?.name}`}</h2>
        <h3>{downloadState.stage === "Downloading" ? `Download Rate: ${readableByteSize(downloadRate)}/s` : '...'}</h3>
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