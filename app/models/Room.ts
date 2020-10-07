import mongoose, {Document, Schema} from "mongoose";
import {makeKey} from "../utilities";

export interface IRoomDocument extends Document {
    key: string;
    name: string;
}

export interface IRoom extends IRoomDocument{
    exportData(): IRoomDocument;
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

RoomSchema.methods.exportData = function (this: IRoomDocument) {
    return {
        key: this.key,
        name: this.name,
    }
}

const Room = mongoose.model<IRoom>('Room', RoomSchema);

export default Room;