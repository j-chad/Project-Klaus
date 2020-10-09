const config = {
	local: {
		mode: "local",
		port: 5000,
		secret: process.env.SERVER_SECRET_KEY
	},
	prod: {
		mode: "prod",
		port: 8080
	}
};

let exportedConfig;
if (process.env.NODE_ENV?.toLowerCase() === "development"){
	exportedConfig = config.local;
} else {
	exportedConfig = config.prod;
}

export default exportedConfig;