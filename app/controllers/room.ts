import express from "express";
import RoomModel from "../models/Room"

const router = express.Router();

router.post('/room', async function (req, res) {
    let name = req.body.name;
    let newRoom = new RoomModel({"name": name});

    await newRoom.save()
    res.send(newRoom.toJSON());
});

export default router;