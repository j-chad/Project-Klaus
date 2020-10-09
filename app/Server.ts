import express, {Express} from "express";
import * as path from "path";

import apiRouter from "./controllers/api";
import mongoose from "mongoose";
import {validateUser} from "./middleware/UserMiddleware";
import cookieParser from "cookie-parser";
import morgan from "morgan";
import logger from "./config/logging";
import config from "./config/config";
import ErrorHandler from "./middleware/ErrorHandlerMiddleware";

export default class Server {

	private app: Express;

	constructor(app: Express) {
		this.app = app;

		this.app.use(express.json());
		this.app.use(cookieParser());
		this.app.use(validateUser);
		this.app.use(morgan('combined', {stream: {write: message => logger.http(message)}}));

		this.app.use(express.static(path.resolve("./") + "/build/frontend"));
		this.app.use("/api/v1/", apiRouter);

		this.app.use(ErrorHandler);

		this.app.get("*", (req, res) => {
			res.sendFile(path.resolve("./") + "/build/frontend/index.html");
		});
	}

	start(port) {
		mongoose.connect(config.db, {
			useNewUrlParser: true,
			useUnifiedTopology: true,
			useCreateIndex: true
		}).then(()=>{
			logger.verbose("Database Connected");
		}).catch((e)=>{
			logger.error(e);
		});
		this.app.listen(port, ()=>{
			logger.info(`Server listening on port ${port}!`);
		});
	}
}