const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

const appDir = path.resolve(__dirname, 'app');
const distDir = path.resolve(__dirname, 'dist');

module.exports = (env, argv) => {
    return {
        mode: 'development',
        entry: {
            app: path.resolve(appDir, 'index.js'),
        },
        output: {
            publicPath: './',
            path: distDir,
            filename: '[name].[contenthash].js',
        },
        devServer: {
            contentBase: distDir,
            port: 8080,
        },
        plugins: [
            new WasmPackPlugin({
                crateDirectory: appDir,
                extraArgs: '--no-typescript',
                outName: 'index',
                outDir: path.join(appDir, 'pkg'),
            }),
            new HtmlWebpackPlugin({
                filename: 'index.html',
                template: path.resolve(__dirname, 'index.html'),
            }),
        ],
        module: {
            rules: [
                {
                    test: /\.css$/i,
                    use: [ 'style-loader', 'css-loader' ],
                }
            ],
        }
    };
};
