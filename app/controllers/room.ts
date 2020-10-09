import express from "express";
import RoomModel from "../models/Room"
import {APIResponse, APIResponseStatus} from "../helpers/APIUtilities";

const router = express.Router();

router.post('/room', async function (req, res) {
    let name = req.body.name;
    let newRoom = new RoomModel({"name": name});

    await newRoom.save()

    res.json(APIResponse(APIResponseStatus.Success, newRoom.exportData()));
});

router.get('/room/:roomKey', async function (req, res) {
    let room = await RoomModel.findOne({key: req.params.roomKey});

    if (room === null){
        res.json(APIResponse(APIResponseStatus.Fail, {
            exists: false,
            room: null
        }))
    } else {
        res.json(APIResponse(APIResponseStatus.Success, {
            exists: true,
            room: room.exportData()
        }));
    }
});

export default router;