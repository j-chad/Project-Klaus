
import {Express, Request, Response} from "express";

export class Server {
    constructor(app) {
        this.app = app;

        this.app.get("/api", (req, res) => {
            res.send("You have reached the API!");
        });
    }

    start(port) {
        this.app.listen(port, () => console.log(`Server listening on port ${port}!`));
    }
}

export default Server;
