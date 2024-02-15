import { useEffect } from 'react';
import { useErrorBoundary } from 'react-error-boundary';

export default function FallbackResetBoundary() {
  const errorBoundary = useErrorBoundary();

  useEffect(() => {
    errorBoundary.resetBoundary();
  }, [errorBoundary]);

  return null;
}
