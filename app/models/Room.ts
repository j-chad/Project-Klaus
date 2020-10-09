import mongoose, {Document, Schema} from "mongoose";
import {makeKey} from "../helpers/utilities";

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
        maxlength: 8,
        minlength: 8,
        match: /^[a-zA-Z0-9]+$/,
        default: function () {
            return makeKey(8);
        }
    },
    name: {
        type: String,
        required: true,
        match: [/^[a-zA-Z0-9 ]+$/, "name can only be letters and numbers"],
        minlength: 4,
        maxlength: 25
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