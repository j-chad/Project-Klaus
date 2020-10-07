import React from "react";
import {Button, ButtonGroup} from "reactstrap";

class CreateRoomButton extends React.Component {
    render() {
        return (
            <ButtonGroup size="lg" className="w-100">
                <Button className="mt-0 mb-0" style={{flexGrow: 1}} disabled={this.props.loading} color="info" onClick={() => {this.props.onModeSwitch(true)}}>Cancel</Button>
                <Button className="mt-0 mb-0" style={{flexGrow: 1}} disabled={!this.props.valid||this.props.loading} onClick={this.props.onCreateRoom} color="primary">Create</Button>
            </ButtonGroup>
        );
    }
}

export default CreateRoomButton;