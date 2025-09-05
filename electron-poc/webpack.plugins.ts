import type IForkTsCheckerWebpackPlugin from 'fork-ts-checker-webpack-plugin';

// eslint-disable-next-line @typescript-eslint/no-var-requires
const ForkTsCheckerWebpackPlugin: typeof IForkTsCheckerWebpackPlugin = require('fork-ts-checker-webpack-plugin');
const webpack = require('webpack');

// CRITICAL: Always read version from package.json - NEVER hardcode!
const packageJson = require('./package.json');

export const plugins = [
  new webpack.DefinePlugin({
    APP_VERSION: JSON.stringify(packageJson.version),
  }),
  new ForkTsCheckerWebpackPlugin({
    logger: 'webpack-infrastructure',
  }),
];
