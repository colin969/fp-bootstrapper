import { Box, Button } from "@mui/material";

export type FailurePageProps = {
  failure: string
};

export function FailurePage(props: FailurePageProps) {
  return (
    <>
      <h1>FATAL ERROR: {props.failure}</h1>
      <h3>Need Help?</h3>
      <Box className='box-row'>
        <Button variant="contained">
          Online FAQ
        </Button>
        <Button variant="contained">
          Discord Server
        </Button>
      </Box>
    </>
  );
}