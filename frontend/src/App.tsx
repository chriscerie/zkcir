import { BrowserRouter, Routes, Route } from 'react-router-dom';

import Header from './components/Header';
import Dashboard from './routes/Dashboard';
import Circuits from './routes/Circuits';
import Auth from './components/Auth';

import { ThemeProvider } from '@mui/material/styles';
import theme from './theme';
import './App.css';
import { UserProvider } from './UserContext';
import { Container } from '@mantine/core';
import NotFound from './routes/NotFound';

function App() {
  return (
    <UserProvider>
      <ThemeProvider theme={theme}>
        <BrowserRouter>
          <Header />
          <Container size="lg">
            <Routes>
              <Route path="/" Component={Dashboard} />
              <Route path="/circuits" element={<Circuits />} />
              <Route path="/auth/callback" element={<Auth />} />
              <Route path="*" Component={NotFound} />
            </Routes>
          </Container>
        </BrowserRouter>
      </ThemeProvider>
    </UserProvider>
  );
}

export default App;
