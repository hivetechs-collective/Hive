const webpack = require('webpack');
const path = require('path');
const fs = require('fs');

class BuildMemoryServicePlugin {
  constructor() {
    this.hasBuilt = false; // Flag to prevent multiple builds
  }
  
  apply(compiler) {
    compiler.hooks.beforeCompile.tapAsync('BuildMemoryServicePlugin', (params, callback) => {
      // Check if we've already built the Memory Service
      if (this.hasBuilt) {
        console.log('[BuildMemoryServicePlugin] Memory Service already built, skipping...');
        return callback();
      }
      
      console.log('[BuildMemoryServicePlugin] Building Memory Service...');
      
      const config = {
        mode: compiler.options.mode || 'development',
        target: 'node',
        entry: path.resolve(compiler.context, 'src', 'memory-service', 'index.ts'),
        output: {
          path: path.resolve(compiler.options.output.path, 'memory-service'),
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
        // Bundle everything - no external dependencies in production
        externals: {},
        node: {
          __dirname: false,
          __filename: false,
        },
      };

      // Ensure output directory exists
      const outputDir = config.output.path;
      if (!fs.existsSync(outputDir)) {
        fs.mkdirSync(outputDir, { recursive: true });
      }

      // Build memory-service
      webpack(config, (err, stats) => {
        if (err) {
          console.error('[BuildMemoryServicePlugin] Build failed:', err);
          return callback(err);
        }
        
        if (stats.hasErrors()) {
          const errors = stats.compilation.errors;
          console.error('[BuildMemoryServicePlugin] Build failed with errors:', errors);
          return callback(errors[0]);
        }
        
        console.log('[BuildMemoryServicePlugin] Memory Service built successfully!');
        this.hasBuilt = true; // Mark as built to prevent re-builds
        callback();
      });
    });
  }
}

module.exports = BuildMemoryServicePlugin;