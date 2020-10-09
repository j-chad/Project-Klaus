import dotenv from "dotenv";
dotenv.config();

import Server from "./app/Server";
import express from 'express';
import config from "./app/config/config";

const app = express();

const server = new Server(app);
server.start(config.port);