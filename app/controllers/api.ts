import express from "express";
import roomRouter from "./room";

const router = express.Router();

router.use("/", roomRouter);

export default router;