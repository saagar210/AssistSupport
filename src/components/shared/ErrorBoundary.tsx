import { Component, ReactNode } from 'react';
import { Button } from './Button';
import './ErrorBoundary.css';

interface ErrorBoundaryProps {
  children: ReactNode;
  fallbackTitle?: string;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo);
  }

  handleRetry = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className="error-boundary">
          <div className="error-boundary-content">
            <div className="error-icon">!</div>
            <h3>{this.props.fallbackTitle || 'Something went wrong'}</h3>
            <p className="error-message">
              {this.state.error?.message || 'An unexpected error occurred.'}
            </p>
            <div className="error-actions">
              <Button variant="primary" onClick={this.handleRetry}>
                Try Again
              </Button>
            </div>
            <details className="error-details">
              <summary>Technical Details</summary>
              <pre>{this.state.error?.stack}</pre>
            </details>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
