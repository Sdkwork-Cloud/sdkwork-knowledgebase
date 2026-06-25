import type { ServerResponse } from 'node:http';

const BASE_SECURITY_HEADERS: Record<string, string> = {
  'X-Content-Type-Options': 'nosniff',
  'X-Frame-Options': 'DENY',
  'Referrer-Policy': 'strict-origin-when-cross-origin',
  'Permissions-Policy': 'camera=(), microphone=(), geolocation=()',
};

function buildContentSecurityPolicy(isDev: boolean): string {
  const connectSrc = isDev
    ? "'self' ws: wss: http://127.0.0.1:* http://localhost:* https:"
    : "'self' https:";
  return [
    "default-src 'self'",
    "base-uri 'self'",
    "object-src 'none'",
    "frame-ancestors 'none'",
    isDev
      ? "script-src 'self' 'unsafe-inline' 'unsafe-eval'"
      : "script-src 'self'",
    "style-src 'self' 'unsafe-inline'",
    "img-src 'self' data: blob: https:",
    "font-src 'self' data:",
    "frame-src 'self' https: blob:",
    `connect-src ${connectSrc}`,
    "worker-src 'self' blob:",
  ].join('; ');
}

function applySecurityHeaders(res: ServerResponse, isDev: boolean): void {
  for (const [name, value] of Object.entries(BASE_SECURITY_HEADERS)) {
    res.setHeader(name, value);
  }
  res.setHeader('Content-Security-Policy', buildContentSecurityPolicy(isDev));
}

export function browserSecurityHeadersPlugin(isDev: boolean) {
  return {
    name: 'sdkwork-knowledgebase-browser-security-headers',
    configureServer(server: { middlewares: { use: (fn: (req: unknown, res: ServerResponse, next: () => void) => void) => void } }) {
      server.middlewares.use((_req, res, next) => {
        applySecurityHeaders(res, true);
        next();
      });
    },
    configurePreviewServer(server: { middlewares: { use: (fn: (req: unknown, res: ServerResponse, next: () => void) => void) => void } }) {
      server.middlewares.use((_req, res, next) => {
        applySecurityHeaders(res, isDev);
        next();
      });
    },
  };
}
