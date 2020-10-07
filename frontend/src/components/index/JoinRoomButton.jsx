import React from "react";
import {Button} from "reactstrap";

class JoinRoomButton extends React.Component {
    render() {
        if (this.props.roomCode.length === 0){
            return <Button color="primary" disabled={this.props.loading} block={true} size="lg" onClick={()=>{this.props.onModeSwitch(false)}}>Create Room</Button>;
        } else if (this.props.roomCode.length === 8) {
            return <Button color="success" onClick={this.props.onClick} disabled={this.props.loading} block={true} size="lg">Join Room</Button>;
        } else {
            return <Button color="info" disabled={true} block={true} size="lg">Join Room</Button>;
        }
    }
}

export default JoinRoomButton;