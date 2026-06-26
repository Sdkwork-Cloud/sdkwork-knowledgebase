import { defineConfig } from 'vitest/config';
import { vitestSharedAliases } from '../../vitest.shared';

export default defineConfig({
  resolve: {
    alias: {
      ...vitestSharedAliases,
    },
  },
  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.ts'],
  },
});
