import { Routes, Route, useNavigate } from 'react-router-dom';

import Header from './components/Header';
import JsonView from './routes/JsonView';
import Home from './routes/Home';
import Auth from './components/Auth';
import { ThemeProvider } from '@mui/material/styles';
import theme from './theme';
import './App.css';
import NotFound from './routes/NotFound';
import NewCircuit from './routes/NewCircuit';
import { ErrorBoundary } from 'react-error-boundary';
import FallbackResetBoundary from './components/FallbackResetBoundary';
import Repo from './routes/Repo';
import { AppShell } from '@mantine/core';
import 'allotment/dist/style.css';
import Settings from './routes/Settings';

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
        <AppShell
          header={{ height: 90 }}
          navbar={{ width: 70, breakpoint: 'sm' }}
        >
          <Header />
          <Routes>
            <Route path="/" Component={Home} />
            <Route path="/json-ir" Component={JsonView} />
            <Route path="/new-circuit" Component={NewCircuit} />
            <Route path="/auth/callback" Component={Auth} />
            <Route path="/settings" Component={Settings} />
            <Route path="/:owner/:repo" Component={Repo} />
            <Route path="*" Component={NotFound} />
          </Routes>
        </AppShell>
      </ThemeProvider>
    </ErrorBoundary>
  );
}

export default App;
