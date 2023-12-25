const CopyWebpackPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const path = require("path");

module.exports = {
	entry: "./bootstrap.js",
	output: {
		path: path.resolve(__dirname, "dist"),
		filename: "bootstrap.js",
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
