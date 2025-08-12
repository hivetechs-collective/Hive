import type { Configuration } from 'webpack';
import MonacoWebpackPlugin from 'monaco-editor-webpack-plugin';
import CopyWebpackPlugin from 'copy-webpack-plugin';

import { rules } from './webpack.rules';
import { plugins } from './webpack.plugins';

rules.push({
  test: /\.css$/,
  use: [{ loader: 'style-loader' }, { loader: 'css-loader' }],
});

// Add Monaco plugin with worker fix
plugins.push(new MonacoWebpackPlugin({
  languages: ['javascript', 'typescript', 'css', 'html', 'json'],
  features: ['!gotoSymbol'],
  globalAPI: true,
}));

// Add CopyWebpackPlugin to serve static files
plugins.push(new CopyWebpackPlugin({
  patterns: [
    { from: 'public', to: '.', noErrorOnMissing: false }
  ],
}));

export const rendererConfig: Configuration = {
  module: {
    rules,
  },
  plugins,
  resolve: {
    extensions: ['.js', '.ts', '.jsx', '.tsx', '.css'],
    fallback: {
      path: false,
      fs: false,
    },
  },
  devServer: {
    client: {
      overlay: false, // Disable the error overlay
    },
  },
  // Suppress the critical dependency warning from Monaco
  ignoreWarnings: [
    {
      module: /monaco-editor/,
      message: /Critical dependency/
    }
  ],
  stats: {
    warningsFilter: [
      /Critical dependency: require function is used in a way in which dependencies cannot be statically extracted/,
    ],
  },
};
