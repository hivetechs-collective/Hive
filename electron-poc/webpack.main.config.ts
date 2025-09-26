import type { Configuration } from 'webpack';
import CopyWebpackPlugin from 'copy-webpack-plugin';
const BuildMemoryServicePlugin = require('./webpack-plugins/BuildMemoryServicePlugin');
const FixBinaryPermissionsPlugin = require('./webpack-plugins/FixBinaryPermissionsPlugin');

import { rules } from './webpack.rules';
import { plugins } from './webpack.plugins';

// Add plugin to build Memory Service during webpack compilation
plugins.push(new BuildMemoryServicePlugin());

// Add plugin to fix binary permissions (Python and Backend Server) after copying
plugins.push(new FixBinaryPermissionsPlugin());

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
        ignore: ['**/__pycache__/**', '**/*.pyc']
      }
    },
    // Copy startup files for main process
    { from: 'startup.html', to: 'startup.html' },
    { from: 'startup-neural.js', to: 'startup-neural.js' },
    { from: 'startup-preload.js', to: 'startup-preload.js' },
    // Help is now integrated into the main window via TypeScript component
    // Copy MEMORY.md guide for AI CLI tools
    { from: 'resources/MEMORY.md', to: 'resources/MEMORY.md' },
    // Copy .env.production with discovered Node.js path (if exists)
    {
      from: '.env.production',
      to: '.env.production',
      noErrorOnMissing: true  // Don't fail if file doesn't exist
    },
    // Copy CLI tool wrappers (like Cline wrapper that fixes VS Code API issues)
    {
      from: 'src/cli-tools',
      to: 'cli-tools',  // Copy to root of output dir, not nested in src
      noErrorOnMissing: true,
      filter: (resourcePath) => {
        // Only copy .js wrapper files
        return resourcePath.endsWith('.js');
      }
    }
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

export default mainConfig;
