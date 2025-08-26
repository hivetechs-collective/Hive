import type { Configuration } from 'webpack';
import CopyWebpackPlugin from 'copy-webpack-plugin';
const BuildMemoryServicePlugin = require('./webpack-plugins/BuildMemoryServicePlugin');
const FixPythonPermissionsPlugin = require('./webpack-plugins/FixPythonPermissionsPlugin');

import { rules } from './webpack.rules';
import { plugins } from './webpack.plugins';

// Add plugin to build Memory Service during webpack compilation
plugins.push(new BuildMemoryServicePlugin());

// Add plugin to fix Python binary permissions after copying
plugins.push(new FixPythonPermissionsPlugin());

// Add CopyWebpackPlugin to copy binaries and startup files
plugins.push(new CopyWebpackPlugin({
  patterns: [
    // Copy backend server binary (preserve execute permissions)
    {
      from: 'binaries',
      to: 'binaries',
      noErrorOnMissing: false,
      // CRITICAL: Preserve execute permissions on binary files
      transform(content, path) {
        return content;
      },
      // Mark binary as executable after copy
      info: {
        minimized: false
      }
    },
    // Copy Python runtime for AI Helpers (preserve structure and permissions)
    {
      from: 'resources/python-runtime',
      to: 'resources/python-runtime',
      noErrorOnMissing: false,
      // Preserve execute permissions on Python binaries
      globOptions: {
        dot: true,
        gitignore: false,
        ignore: ['**/__pycache__/**', '**/*.pyc']
      }
    },
    // Copy startup files for main process
    { from: 'startup.html', to: 'startup.html' },
    { from: 'startup-neural.js', to: 'startup-neural.js' },
    { from: 'startup-preload.js', to: 'startup-preload.js' }
  ],
}));

export const mainConfig: Configuration = {
  /**
   * This is the main entry point for your application, it's the first file
   * that runs in the main process.
   */
  entry: './src/index.ts',
  // Put your normal webpack config below here
  module: {
    rules,
  },
  plugins,
  resolve: {
    extensions: ['.js', '.ts', '.jsx', '.tsx', '.css', '.json'],
  },
  // CRITICAL: Preserve stdio configuration strings during minification
  optimization: {
    minimize: true,
    minimizer: [
      // Use default minimizer but preserve critical strings
      '...',
    ],
    // Prevent aggressive optimizations that could break stdio config
    usedExports: true,
    sideEffects: false,
    // Keep critical function names for debugging
    mangleExports: false,
  },
  // Prevent webpack from watching .git directory
  watchOptions: {
    ignored: [
      '**/node_modules',
      '**/.git/**',
      '**/dist/**',
      '**/.webpack/**'
    ],
  },
  // Mark node-pty as external so webpack doesn't try to bundle it
  externals: {
    'node-pty': 'commonjs node-pty'
  }
};
