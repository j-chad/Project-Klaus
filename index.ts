import Server from "./app/Server";
import express from 'express';
import dotenv from "dotenv";

import config from "./app/config/config"

dotenv.config();

const app = express();

const port = 8080;

const server = new Server(app);
server.start(config.port);