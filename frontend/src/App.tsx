import { BrowserRouter, Routes, Route } from 'react-router-dom';

import Header from './components/Header';
import JsonView from './routes/JsonView';
import Home from './routes/Home';
import Auth from './components/Auth';
import { ThemeProvider } from '@mui/material/styles';
import theme from './theme';
import './App.css';
import { UserProvider } from './UserContext';
import { Container } from '@mantine/core';
import NotFound from './routes/NotFound';
import NewCircuit from './routes/NewCircuit';
import { QueryClient, QueryClientProvider } from 'react-query';

const queryClient = new QueryClient();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <UserProvider>
        <ThemeProvider theme={theme}>
          <BrowserRouter>
            <Header />
            <Container size="lg">
              <Routes>
                <Route path="/" Component={Home} />
                <Route path="/json-ir" Component={JsonView} />
                <Route path="/new-circuit" Component={NewCircuit} />
                <Route path="/auth/callback" Component={Auth} />
                <Route path="*" Component={NotFound} />
              </Routes>
            </Container>
          </BrowserRouter>
        </ThemeProvider>
      </UserProvider>
    </QueryClientProvider>
  );
}

export default App;
