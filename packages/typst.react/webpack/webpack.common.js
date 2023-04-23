const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');

module.exports = mod => {
  return {
    entry:
      mod === 'dev'
        ? path.resolve(__dirname, '..', './src/demo/index.tsx')
        : path.resolve(__dirname, '..', './src/lib/index.tsx'),
    resolve: {
      extensions: ['.tsx', '.ts', '.js'],
    },
    externals: {
      /// good related issue: https://github.com/wasmerio/wasmer-js/issues/290
      'typst_renderer_ts_bg.wasm': true,
    },
    module: {
      rules: [
        {
          test: /\.(ts|js)x?$/,
          exclude: /node_modules/,
          use: [
            {
              loader: 'babel-loader',
            },
          ],
        },
        {
          test: /\.css$/,
          use: ['style-loader', 'css-loader'],
        },
        {
          test: /\.(?:ico|gif|png|jpg|jpeg)$/i,
          type: 'asset/resource',
        },
        {
          test: /\.(woff(2)?|eot|ttf|otf|svg|)$/,
          type: 'asset/inline',
        },
      ],
    },
    output: {
      hashFunction: 'xxhash64',
      path: path.resolve(__dirname, '..', './dist'),
      filename: 'main.js',
    },
    plugins: [
      ...(mod === 'dev'
        ? [
            new HtmlWebpackPlugin({
              template: path.resolve(__dirname, '..', './src/demo/index.html'),
            }),
          ]
        : []),
    ],
    stats: 'errors-only',
  };
};
