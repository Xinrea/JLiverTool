const path = require('path');
const webpack = require("webpack");

module.exports = {
  mode: "development",
  entry: {
    './src/gift-window/gift-window.min': './src/gift-window/gift-window.ts',
    './src/superchat-window/superchat-window.min': './src/superchat-window/superchat-window.ts',
    './src/main-window/main-window.min': './src/main-window/main-window.ts'
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
    ],
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
  },
  output: {
    filename: '[name].js',
    path: path.resolve(__dirname, './'),
  },
  plugins: [
    new webpack.IgnorePlugin({
      resourceRegExp: /^\.\/locale$/,
      contextRegExp: /moment$/,
    })
  ],
};