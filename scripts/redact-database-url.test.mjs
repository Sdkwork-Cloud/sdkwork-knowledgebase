import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { redactDatabaseUrl } from './lib/redact-database-url.mjs';

describe('database URL redaction', () => {
  it('removes usernames and passwords while retaining routing context', () => {
    const redacted = redactDatabaseUrl(
      'postgresql://knowledge_user:secret-value@db.example.com:5432/knowledge?sslmode=require',
    );

    assert.doesNotMatch(redacted, /knowledge_user|secret-value/);
    assert.match(redacted, /db\.example\.com:5432\/knowledge/);
    assert.match(redacted, /sslmode=require/);
  });

  it('does not echo malformed or missing values', () => {
    assert.equal(redactDatabaseUrl('not a database URL'), '<invalid-database-url>');
    assert.equal(redactDatabaseUrl(undefined), 'unknown');
  });
});
