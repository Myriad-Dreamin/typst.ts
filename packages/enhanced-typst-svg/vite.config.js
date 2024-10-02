module.exports = {
  build: {
    emptyOutDir: false,
    lib: {
      formats: ['cjs'],
      entry: 'src/index.ts',
      fileName: 'index.min',
    },
  },
};
