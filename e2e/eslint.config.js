import tseslint from 'typescript-eslint';

export default tseslint.config(...tseslint.configs.recommended, {
  rules: {
    // Console output is intentional in test fixtures for verbose debugging
    'no-console': 'off',
    // Empty catch blocks are intentional in test utilities (e.g. retry loops, cleanup)
    'no-empty': ['error', { allowEmptyCatch: true }],
  },
});
