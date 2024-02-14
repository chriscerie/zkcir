import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Box from '@mui/material/Box';
import logo from '../assets/logos.png';
import { Typography } from '@mui/material';
import { Authenticator } from '@aws-amplify/ui-react';
import '@aws-amplify/ui-react/styles.css';

// eslint-disable-next-line @typescript-eslint/no-unused-vars -- REMOVE ME WHEN `onLogin` IS USED
function Login({ onLogin }: { onLogin: () => void }) {
  return (
    <Box
      sx={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        height: '100vh',
      }}
    >
      <Card
        sx={{ width: '33%', alignItems: 'center', justifyContent: 'center' }}
      >
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            marginRight: '9%',
          }}
        >
          <img src={logo} alt="logo" width="30%" />
          <Typography
            variant="h2"
            sx={{ fontFamily: 'Orbitron, Arial, sans-serif' }}
          >
            LOGOS
          </Typography>
        </Box>

        <CardContent
          sx={{
            margin: '1%',
          }}
        >
          <Authenticator>
            {/* {({ signOut, user }) => (
                            <div>
                                <p>Welcome {user.username}</p>
                                <button onClick={signOut}>Sign out</button>
                            </div>
                        )} */}
          </Authenticator>
          {/* <TextField label="Email" variant="outlined" type="email" fullWidth />
                    <TextField label="Password" variant="outlined" type="password" fullWidth />
                    <Button variant="contained" color="primary" sx={{ mt: 2 }}>
                        Login
                    </Button> */}
        </CardContent>
      </Card>
    </Box>
  );
}

export default Login;
