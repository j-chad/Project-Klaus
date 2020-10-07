import express from "express";
import RoomModel from "../models/Room"
import {APIResponse, APIResponseStatus} from "../APIUtilities";
import {Document} from "mongoose";

const router = express.Router();

router.post('/room', async function (req, res) {
    let name = req.body.name;
    let newRoom = new RoomModel({"name": name});

    await newRoom.save()

    let responseData = {
        key: newRoom.key,
        name: newRoom.name
    };
    res.send(APIResponse(APIResponseStatus.Success, responseData));
});

export default router;