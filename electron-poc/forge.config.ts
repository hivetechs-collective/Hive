import type { ForgeConfig } from '@electron-forge/shared-types';
import { MakerSquirrel } from '@electron-forge/maker-squirrel';
import { MakerZIP } from '@electron-forge/maker-zip';
import { MakerDMG } from '@electron-forge/maker-dmg';
import { MakerDeb } from '@electron-forge/maker-deb';
import { MakerRpm } from '@electron-forge/maker-rpm';
import { AutoUnpackNativesPlugin } from '@electron-forge/plugin-auto-unpack-natives';
import { WebpackPlugin } from '@electron-forge/plugin-webpack';
import { FusesPlugin } from '@electron-forge/plugin-fuses';
import { FuseV1Options, FuseVersion } from '@electron/fuses';

import { mainConfig } from './webpack.main.config';
import { rendererConfig } from './webpack.renderer.config';

const config: ForgeConfig = {
  packagerConfig: {
    asar: {
      unpack: '**/{*.node,node_modules/node-pty/**,node_modules/better-sqlite3/**,node_modules/sqlite3/**,.webpack/main/binaries/**,.webpack/main/resources/python-runtime/**,.webpack/main/memory-service/**}'
    },
    icon: './resources/icon', // Don't include file extension
    name: 'Hive Consensus',
    appBundleId: 'com.hivetechs.consensus',
    appCategoryType: 'public.app-category.developer-tools',
  },
  rebuildConfig: {},
  makers: [
    new MakerSquirrel({
      setupIcon: './resources/icon.ico'
    }), 
    new MakerZIP({}, ['darwin']), 
    new MakerDMG({
      format: 'ULFO',
      icon: './resources/icon.icns',
      name: 'Hive Consensus'
    }),
    new MakerRpm({}), 
    new MakerDeb({
      options: {
        maintainer: 'HiveTechs',
        homepage: 'https://hivetechs.io',
        categories: ['Development']
      }
    })
  ],
  
  hooks: {
    async preMake() {
      const fs = require('fs');
      const path = require('path');
      const outDir = path.resolve(__dirname, 'out');
      const candidates = [
        path.join(outDir, 'Hive Consensus-darwin-arm64', 'Hive Consensus.app'),
        path.join(outDir, 'Hive Consensus-darwin-x64', 'Hive Consensus.app'),
      ];
      for (const appPath of candidates) {
        const nodeP = path.join(appPath, 'Contents/Resources/app.asar.unpacked/.webpack/main/binaries/node');
        const gitP = path.join(appPath, 'Contents/Resources/app.asar.unpacked/.webpack/main/binaries/git-bundle/bin/git');
        for (const [label, pth] of [['node', nodeP], ['git', gitP]]) {
          try {
            if (fs.existsSync(pth)) {
              fs.chmodSync(pth, 0o755);
              console.log(`[forge preMake] chmod 755 ${label}: ${pth}`);
            }
          } catch (e) {
            console.log(`[forge preMake] chmod failed for ${label}: ${e.message}`);
          }
        }
      }
    }
  },

  plugins: [
    new AutoUnpackNativesPlugin({}),
    new WebpackPlugin({
      mainConfig,
      renderer: {
        config: rendererConfig,
        entryPoints: [
          {
            html: './src/index.html',
            js: './src/renderer.ts',
            name: 'main_window',
            preload: {
              js: './src/preload.ts',
            },
          },
        ],
      },
      port: 9100, // Use port 9100 instead of default 9000 to avoid conflicts
      loggerPort: 9101, // Logger port also needs to be different
    }),
    // Fuses are used to enable/disable various Electron functionality
    // at package time, before code signing the application
    new FusesPlugin({
      version: FuseVersion.V1,
      [FuseV1Options.RunAsNode]: false,
      [FuseV1Options.EnableCookieEncryption]: true,
      [FuseV1Options.EnableNodeOptionsEnvironmentVariable]: false,
      [FuseV1Options.EnableNodeCliInspectArguments]: false,
      [FuseV1Options.EnableEmbeddedAsarIntegrityValidation]: true,
      [FuseV1Options.OnlyLoadAppFromAsar]: true,
    }),
  ],
};

export default config;
