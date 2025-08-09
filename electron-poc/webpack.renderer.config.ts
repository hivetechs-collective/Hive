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
};
