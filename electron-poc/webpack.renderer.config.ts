import type { Configuration } from 'webpack';
import MonacoWebpackPlugin from 'monaco-editor-webpack-plugin';
import CopyWebpackPlugin from 'copy-webpack-plugin';
import MiniCssExtractPlugin from 'mini-css-extract-plugin';

import { rules } from './webpack.rules';
import { plugins } from './webpack.plugins';

// Use style-loader for development, MiniCssExtractPlugin for production
const isDevelopment = process.env.NODE_ENV !== 'production';

rules.push({
  test: /\.css$/,
  use: [
    isDevelopment ? 'style-loader' : MiniCssExtractPlugin.loader,
    { loader: 'css-loader' }
  ],
});

// Add MiniCssExtractPlugin for production builds
if (!isDevelopment) {
  plugins.push(new MiniCssExtractPlugin({
    filename: '[name].css',
    chunkFilename: '[id].css',
  }));
}

// Add Monaco plugin with worker fix
plugins.push(new MonacoWebpackPlugin({
  languages: ['javascript', 'typescript', 'css', 'html', 'json'],
  features: ['!gotoSymbol'],
  globalAPI: true,
}));

// Add CopyWebpackPlugin to serve static files
plugins.push(new CopyWebpackPlugin({
  patterns: [
    { from: 'public', to: '.', noErrorOnMissing: false },
    // Copy startup files for splash screen
    { from: 'startup.html', to: 'startup.html' },
    { from: 'startup-neural.js', to: 'startup-neural.js' },
    { from: 'startup-preload.js', to: 'startup-preload.js' },
    // Copy AI CLI tool icons
    { from: 'resources/ai-cli-icons', to: 'resources/ai-cli-icons' },
    // Copy app icon
    { from: 'resources/icon.png', to: 'resources/icon.png' }
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
  watchOptions: {
    // Ignore .git directory and node_modules to prevent reloads
    ignored: [
      '**/node_modules',
      '**/.git/**',
      '**/dist/**',
      '**/.webpack/**'
    ],
  },
  devServer: {
    hot: false, // Completely disable hot module replacement
    liveReload: false, // Disable live reload
    client: {
      overlay: false, // Disable the error overlay
      reconnect: false, // Don't auto-reconnect on connection loss
    },
    // Disable hot reload for .git changes
    watchFiles: {
      paths: ['src/**/*', 'public/**/*'],
      options: {
        ignored: ['**/node_modules', '**/.git/**', '**/dist/**', '**/.webpack/**'],
      },
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
