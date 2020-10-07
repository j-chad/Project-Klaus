
import express from "express";
import * as path from "path";

export class Server {
    constructor(app) {
        this.app = app;

        this.app.use(express.static(path.resolve("./") + "/build/frontend"));

        this.app.get("/api", (req, res) => {
            res.send("You have reached the API!");
        });

        this.app.get("*", (req, res) => {
            res.sendFile(path.resolve("./") + "/build/frontend/index.html");
        });
    }

    start(port) {
        this.app.listen(port, () => console.log(`Server listening on port ${port}!`));
    }
}

export default Server;
