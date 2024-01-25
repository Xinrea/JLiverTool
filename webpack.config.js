const path = require('path')
const webpack = require('webpack')

const frontendConfig = {
  mode: 'production',
  devtool: 'inline-source-map',
  entry: {
    './src/gift-window/gift-window': './src/gift-window/gift-window.ts',
    './src/superchat-window/superchat-window':
      './src/superchat-window/superchat-window.ts',
    './src/main-window/main-window': './src/main-window/main-window.ts',
    './src/setting-window/setting-window':
      './src/setting-window/setting-window.ts',
    './src/detail-window/detail-window': './src/detail-window/detail-window.ts',
    './src/rank-window/rank-window': './src/rank-window/rank-window.ts',
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
    }),
  ],
}

const preloadConfig = {
  mode: 'production',
  devtool: 'inline-source-map',
  target: 'electron-preload',
  entry: {
    './src/preload': './src/preload.ts',
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
    }),
  ],
}

const backendConfig = {
  mode: 'production',
  devtool: 'inline-source-map',
  target: 'electron-main',
  entry: {
    './src/main': './src/main.ts',
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
      {
        test: /\.node$/,
        use: 'node-loader',
      },
    ],
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
    fallback: {
      bufferutil: false,
      'utf-8-validate': false,
    },
  },
  output: {
    filename: '[name].js',
    path: path.resolve(__dirname, './'),
  },
  plugins: [
    new webpack.IgnorePlugin({
      resourceRegExp: /^\.\/locale$/,
      contextRegExp: /moment$/,
    }),
  ],
}

module.exports = [frontendConfig, preloadConfig, backendConfig]
