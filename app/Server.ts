import express, {Express} from "express";
import * as path from "path";

import apiRouter from "./controllers/api";
import mongoose from "mongoose";
import {validateUser} from "./UserMiddleware";
import cookieParser from "cookie-parser";

export default class Server {

	private app: Express;

	constructor(app: Express) {
		this.app = app;

		app.use(express.json());
		app.use(cookieParser())
		app.use(validateUser);

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
		this.app.listen(port, () => console.log(`Server listening on port ${port}!`));
	}
}