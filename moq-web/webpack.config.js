const CopyWebpackPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const path = require("path");

module.exports = {
	entry: "./index.ts",
	output: {
		path: path.resolve(__dirname, "dist"),
		filename: "index.js",
	},
	module: {
		rules: [
			{
				test: /\.tsx?$/,
				use: "ts-loader",
				exclude: /node_modules/,
			},
		],
	},
	resolve: {
		extensions: [".tsx", ".ts", ".js"],
	},
	mode: "development",
	plugins: [
		new CopyWebpackPlugin({ patterns: ["index.html"] }),
		new WasmPackPlugin({
			crateDirectory: path.resolve(__dirname, "."),
		}),
	],
	experiments: {
		asyncWebAssembly: true,
	},
	watchOptions: {
		aggregateTimeout: 200,
		poll: 200,
	},
};
