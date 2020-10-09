import express, {Express} from "express";
import * as path from "path";

import apiRouter from "./controllers/api";
import mongoose from "mongoose";
import {validateUser} from "./middleware/UserMiddleware";
import cookieParser from "cookie-parser";
import morgan from "morgan";
import logger from "./config/logging";

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

		this.app.get("*", (req, res) => {
			res.sendFile(path.resolve("./") + "/build/frontend/index.html");
		});
	}

	start(port) {
		mongoose.connect('mongodb://localhost/klaus', {
			useNewUrlParser: true,
			useUnifiedTopology: true,
			useCreateIndex: true
		}).catch(r => {
			console.error("Error connecting to db");
		});
		this.app.listen(port, () => logger.info(`Server listening on port ${port}!`));
	}
}