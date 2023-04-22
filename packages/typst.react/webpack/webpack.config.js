const { merge } = require('webpack-merge');
const createCommonConfig = require('./webpack.common.js');

module.exports = envVars => {
  const { env } = envVars;
  const envConfig = require(`./webpack.${env}.js`);
  const config = merge(createCommonConfig(env), envConfig);
  return config;
};
