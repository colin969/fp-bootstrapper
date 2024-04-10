import { Box, Button } from "@mui/material";

export function FinishedPage() {
  return (
    <div>
      <h1>Installation Complete!</h1>
      <Box className="row-box">
        <Button variant="contained" color="success">
          Launch Flashpoint
        </Button>
        <Button variant="contained">
          Exit
        </Button>
      </Box>
    </div>
  );
}