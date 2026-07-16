export function redactDatabaseUrl(value) {
  const normalized = typeof value === 'string' ? value.trim() : '';
  if (!normalized) {
    return 'unknown';
  }
  try {
    const parsed = new URL(normalized);
    if (parsed.username) {
      parsed.username = '***';
    }
    if (parsed.password) {
      parsed.password = '***';
    }
    return parsed.toString();
  } catch {
    return '<invalid-database-url>';
  }
}
