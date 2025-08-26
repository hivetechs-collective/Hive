const webpack = require('webpack');
const path = require('path');
const fs = require('fs');

const config = {
  mode: process.env.NODE_ENV === 'production' ? 'production' : 'development',
  target: 'node',
  entry: path.resolve(__dirname, '..', 'src', 'memory-service', 'index.ts'),
  output: {
    path: path.resolve(__dirname, '..', 'dist', 'memory-service'),
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
  },
  node: {
    __dirname: false,
    __filename: false,
  },
};

console.log('Building Memory Service...');

// Ensure output directory exists
const outputDir = path.resolve(__dirname, '..', 'dist', 'memory-service');
if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}

webpack(config, (err, stats) => {
  if (err) {
    console.error('Build failed:', err);
    process.exit(1);
  }
  
  if (stats.hasErrors()) {
    console.error('Build failed with errors:');
    console.error(stats.toString({ colors: true }));
    process.exit(1);
  }
  
  console.log('Memory Service built successfully!');
  console.log(stats.toString({ colors: true, modules: false }));
});