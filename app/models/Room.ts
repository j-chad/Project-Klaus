import mongoose from "mongoose";

const RoomSchema = new mongoose.Schema({
    key: {
        type: String,
        index: true,
        unique: true,
        required: true
    },
    name: {
        type: String,
        required: true
    }
})

const Room = mongoose.model('Room', RoomSchema);

export default Room;