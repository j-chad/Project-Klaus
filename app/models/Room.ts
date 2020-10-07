import mongoose from "mongoose";
import {makeKey} from "../utilities";

const RoomSchema = new mongoose.Schema({
    key: {
        type: String,
        index: true,
        unique: true,
        required: true,
        default: function () {
            return makeKey(8);
        }
    },
    name: {
        type: String,
        required: true,
        minlength: 4
    }
})

const Room = mongoose.model('Room', RoomSchema);

export default Room;