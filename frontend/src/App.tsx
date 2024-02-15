import { Routes, Route, Navigate, useNavigate } from 'react-router-dom';

import Header from './components/Header';
import JsonView from './routes/JsonView';
import Home from './routes/Home';
import Auth from './components/Auth';
import { ThemeProvider } from '@mui/material/styles';
import theme from './theme';
import './App.css';
import { Container } from '@mantine/core';
import NotFound from './routes/NotFound';
import NewCircuit from './routes/NewCircuit';
import { ErrorBoundary } from 'react-error-boundary';
import FallbackResetBoundary from './components/FallbackResetBoundary';

function App() {
  const navigate = useNavigate();

  return (
    <ErrorBoundary
      FallbackComponent={FallbackResetBoundary}
      onReset={() => {
        navigate('/');
      }}
      onError={(e) => alert(`Caught unexpected error: ${e}`)}
    >
      <ThemeProvider theme={theme}>
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
      </ThemeProvider>
    </ErrorBoundary>
  );
}

export default App;
