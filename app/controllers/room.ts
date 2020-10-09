import express from "express";
import RoomModel from "../models/Room"
import {APIResponse, APIResponseStatus} from "../helpers/APIUtilities";
import mongoose from "mongoose";

const router = express.Router();

router.post('/room', async function (req, res, next) {
    let name = req.body.name;
    let newRoom = new RoomModel({"name": name});

    try{
        await newRoom.save()
    } catch(e){
        next(e);
        return;
    }

    res.status(201);
    res.json(APIResponse(APIResponseStatus.Success, newRoom.exportData()));
});

router.get('/room/:roomKey', async function (req, res) {
    let room = await RoomModel.findOne({key: req.params.roomKey});

    if (room === null){
        res.status(404);
        res.json(APIResponse(APIResponseStatus.Fail, {
            exists: false,
            room: null
        }));
    } else {
        res.status(200)
        res.json(APIResponse(APIResponseStatus.Success, {
            exists: true,
            room: room.exportData()
        }));
    }
});

export default router;