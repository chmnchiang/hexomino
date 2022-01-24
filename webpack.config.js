const path = require('path');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');
const CopyWebpackPlugin = require('copy-webpack-plugin');

const distPath = path.resolve(__dirname, "dist");
module.exports = (env, argv) => {
  return {
    devServer: {
      static: distPath,
      compress: argv.mode === 'production',
      port: 8000
    },
    entry: './js/index.js',
    output: {
      path: distPath,
      filename: "index.js",
      webassemblyModuleFilename: "index.wasm"
    },
    plugins: [
      new CopyWebpackPlugin({
        patterns: [
          { from: './static', to: distPath },
        ],
      }),
      new WasmPackPlugin({
        crateDirectory: ".",
        extraArgs: "--no-typescript",
      })
    ],
    watch: argv.mode !== 'production',
    experiments: {
      asyncWebAssembly: true,
    },
  };
};
