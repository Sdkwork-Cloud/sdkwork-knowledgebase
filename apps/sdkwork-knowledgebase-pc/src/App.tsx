import React, { Component, useMemo, type ErrorInfo, type ReactNode } from 'react';
import { BrowserRouter, Route, Routes, useLocation, useNavigate } from 'react-router-dom';
import { AlertTriangle, RefreshCw } from 'lucide-react';
import {
  KnowledgebaseAuthGate,
  KnowledgebaseRuntimeProvider,
} from 'sdkwork-knowledgebase-pc-core';
import { AppShell } from '@packages/sdkwork-knowledgebase-pc-shell/src';
import { WechatPublishPage } from '@packages/sdkwork-knowledgebase-pc-knowledgebase/src/WechatPublishPage';

import { createKnowledgebasePcRuntime } from './bootstrap/createKnowledgebasePcRuntime';
import type { KnowledgebaseIamRuntime } from './bootstrap/knowledgebaseIamRuntime';
import {
  resolveKnowledgebaseAuthAppearance,
  resolveKnowledgebaseAuthLocale,
  resolveKnowledgebaseAuthRuntimeConfig,
} from './bootstrap/knowledgebaseAuthConfig';
import { KnowledgebaseAuthShell } from './components/KnowledgebaseAuthShell';

const SdkworkIamAuthRoutes = React.lazy(() =>
  import('@sdkwork/auth-pc-react').then((module) => ({ default: module.SdkworkIamAuthRoutes })),
);

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  declare readonly props: Readonly<ErrorBoundaryProps>;

  public state: ErrorBoundaryState = {
    hasError: false,
    error: null,
  };

  public static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  public componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Uncaught error:', error, errorInfo);
  }

  public render() {
    if (this.state.hasError) {
      return (
        <div className="w-screen h-screen flex flex-col items-center justify-center bg-gray-50 text-gray-800 p-6">
          <AlertTriangle size={64} className="text-red-500 mb-6" />
          <h1 className="text-2xl font-bold mb-2">Something went wrong.</h1>
          <p className="text-gray-600 mb-6 text-center max-w-md">
            The application encountered an unexpected error. Please try reloading the page or contact support if the issue persists.
          </p>
          <div className="bg-white border rounded shadow-sm p-4 w-full max-w-2xl overflow-auto mb-8">
            <pre className="text-xs text-red-600 font-mono">
              {this.state.error?.message}
            </pre>
          </div>
          <button
            onClick={() => window.location.reload()}
            className="flex items-center px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors shadow"
          >
            <RefreshCw size={18} className="mr-2" />
            Reload Application
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

export default function App() {
  const runtime = useMemo(() => createKnowledgebasePcRuntime(), []);

  return (
    <ErrorBoundary>
      <KnowledgebaseRuntimeProvider runtime={runtime}>
        <BrowserRouter>
          <KnowledgebaseAppRoutes runtime={runtime} />
        </BrowserRouter>
      </KnowledgebaseRuntimeProvider>
    </ErrorBoundary>
  );
}

function KnowledgebaseAppRoutes({
  runtime,
}: {
  runtime: ReturnType<typeof createKnowledgebasePcRuntime>;
}) {
  const location = useLocation();
  const navigate = useNavigate();

  const getIamRuntime = useMemo(() => {
    return () => getKnowledgebaseIamRuntime(runtime);
  }, [runtime]);

  const authRoutes = useMemo(() => (
    <KnowledgebaseAuthShell>
      <KnowledgebaseAppbaseAuthRouteHost getRuntime={getIamRuntime} />
    </KnowledgebaseAuthShell>
  ), [getIamRuntime]);

  return (
    <KnowledgebaseAuthGate
      authRoutes={authRoutes}
      location={location}
      navigate={(to, options) => navigate(to, options)}
      session={runtime.session}
    >
      <Routes>
        <Route path="/" element={<AppShell />} />
        <Route path="/wechat-publish" element={<WechatPublishPage />} />
      </Routes>
    </KnowledgebaseAuthGate>
  );
}

function KnowledgebaseAppbaseAuthRouteHost({
  getRuntime,
}: {
  getRuntime: () => KnowledgebaseIamRuntime;
}) {
  const props = {
    appearance: resolveKnowledgebaseAuthAppearance(),
    basePath: '/auth',
    getRuntime,
    homePath: '/',
    locale: resolveKnowledgebaseAuthLocale(),
    runtimeConfig: resolveKnowledgebaseAuthRuntimeConfig(),
    viewportMode: 'fixed' as const,
  };

  return (
    <React.Suspense fallback={<KnowledgebaseAuthRoutesFallback />}>
      <SdkworkIamAuthRoutes {...props as any} />
    </React.Suspense>
  );
}

function getKnowledgebaseIamRuntime(
  runtime: ReturnType<typeof createKnowledgebasePcRuntime>,
): KnowledgebaseIamRuntime {
  const iamRuntime = runtime.auth?.iamRuntime;
  if (!iamRuntime) {
    throw new Error('Knowledgebase IAM runtime is not configured.');
  }
  return iamRuntime as KnowledgebaseIamRuntime;
}

function KnowledgebaseAuthRoutesFallback() {
  return (
    <div
      aria-label="Loading Knowledgebase auth routes"
      className="sdkwork-knowledgebase-auth-loading"
    >
      <div className="h-7 w-7 rounded-full border-2 border-blue-500 border-t-transparent animate-spin" />
    </div>
  );
}
