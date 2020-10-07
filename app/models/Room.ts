import mongoose, {Document, Schema} from "mongoose";
import {makeKey} from "../utilities";

export interface IRoom extends Document {
    key: string;
    name: string;
}

export const RoomSchema = new Schema({
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

const Room = mongoose.model<IRoom>('Room', RoomSchema);

export default Room;