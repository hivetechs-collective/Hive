import type { Configuration } from 'webpack';
import path from 'path';

export const memoryServiceConfig: Configuration = {
  mode: 'production',
  target: 'node',
  entry: './src/memory-service/index.ts',
  output: {
    path: path.resolve(__dirname, '.webpack', 'memory-service'),
    filename: 'index.js',
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        exclude: /(node_modules|\.webpack)/,
        use: {
          loader: 'ts-loader',
          options: {
            transpileOnly: true,
          },
        },
      },
    ],
  },
  resolve: {
    extensions: ['.js', '.ts', '.json'],
  },
  externals: {
    'express': 'commonjs express',
    'ws': 'commonjs ws',
    'cors': 'commonjs cors',
    'crypto': 'commonjs crypto',
    'http': 'commonjs http',
    'path': 'commonjs path',
  },
  node: {
    __dirname: false,
    __filename: false,
  },
};