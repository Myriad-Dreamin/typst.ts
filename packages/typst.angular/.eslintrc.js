const tsRule = {
  files: ['*.ts'],
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/eslint-recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:@typescript-eslint/recommended-requiring-type-checking',
  ],
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 10,
    project: ['./tsconfig.json'],
    sourceType: 'module',
    tsconfigRootDir: __dirname,
    ecmaFeatures: {
      modules: true,
    },
  },
  plugins: [
    '@typescript-eslint',
    // "@angular-eslint/eslint-plugin"
  ],
  rules: {
    '@typescript-eslint/indent': [
      'error',
      2,
      {
        SwitchCase: 1,
      },
    ],
    '@typescript-eslint/member-delimiter-style': [
      'error',
      {
        multiline: {
          delimiter: 'semi',
          requireLast: true,
        },
        singleline: {
          delimiter: 'semi',
          requireLast: false,
        },
      },
    ],
    semi: [2, 'always'],
    '@typescript-eslint/no-inferrable-types': [
      'error',
      {
        ignoreParameters: true,
        ignoreProperties: true,
      },
    ],
    '@typescript-eslint/no-empty-function': 0,
    '@typescript-eslint/no-var-requires': 0,
    '@typescript-eslint/no-explicit-any': 0,
    '@typescript-eslint/no-floating-promises': 0,
    '@typescript-eslint/no-unsafe-assignment': 0,
    '@typescript-eslint/no-unsafe-return': 0,
    '@typescript-eslint/no-unsafe-call': 0,
    '@typescript-eslint/no-unsafe-member-access': 0,
    '@typescript-eslint/unbound-method': 0,
    // "@angular-eslint/use-injectable-provided-in": "error",
    // "@angular-eslint/no-attribute-decorator": "error"
  },
};

const htmlRule = {
  files: ['*.component.html'],
  extends: ['plugin:@angular-eslint/template/recommended'],
};

// noinspection SpellCheckingInspection
module.exports = {
  env: {
    browser: true,
    node: true,
    es6: true,
    es2015: true,
    es2017: true,
  },
  overrides: [tsRule, htmlRule],
};
