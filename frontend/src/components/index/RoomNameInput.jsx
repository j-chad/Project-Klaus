import React from "react";
import {Input, InputGroupAddon, InputGroupText, InputGroup} from 'reactstrap';

export default class RoomNameInput extends React.Component {
    constructor(props) {
        super(props);
        this.state = {focused: false};

        this.onRoomNameFocus = this.onRoomNameFocus.bind(this);
        this.onRoomNameBlur = this.onRoomNameBlur.bind(this);
        this.onChange = this.onChange.bind(this);
    }

    onRoomNameFocus() {
        this.setState({
            focused: true
        });
    }

    onRoomNameBlur() {
        this.setState({
            focused: false
        });
    }

    onChange(event){
        if (!this.props.disabled){
            this.props.onChange(event);
        }
    }

    render() {
        return (
            <InputGroup size="lg" className={this.state.focused ? "input-group-focus" : ""}>
                <InputGroupAddon addonType="prepend">
                    <InputGroupText>
                        <i className="tim-icons icon-badge"/>
                    </InputGroupText>
                </InputGroupAddon>
                    <Input className="mb-2 text-center text-uppercase"
                           type="text"
                           placeholder="Room Name"
                           minLength={4}
                           maxLength={20}
                           required
                           pattern={"[0-9A-Za-z]*"}
                           bsSize="lg"
                           value={this.props.name}
                           onFocus={this.onRoomNameFocus}
                           onBlur={this.onRoomNameBlur}
                           onChange={this.onChange}
                    />
            </InputGroup>
        );
    }
}

