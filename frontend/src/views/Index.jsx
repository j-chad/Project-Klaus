import React from "react";
import {Container} from "reactstrap";
import "../assets/scss/index.scss";
import logo from "../assets/images/logo.svg";
import JoinRoomButton from "../components/index/JoinRoomButton";
import RoomCodeInput from "../components/index/RoomCodeInput";
import CreateRoomButton from "../components/index/CreateRoomButton";
import RoomNameInput from "../components/index/RoomNameInput";
import {CreateRoom} from "../api/rooms";

class Index extends React.Component {
    constructor(props) {
        super(props);

        this.state = {
            joinRoom: true,
            roomCode: "",
            roomName: "",
            roomNameValid: false,
            loading: false
        };

        this.handleRoomCodeInput = this.handleRoomCodeInput.bind(this);
        this.handleRoomNameInput = this.handleRoomNameInput.bind(this);
        this.handleSwitchMode = this.handleSwitchMode.bind(this);
        this.onCreateRoom = this.onCreateRoom.bind(this);
    }

    handleRoomCodeInput(event) {
        this.setState({roomCode: event.target.value});
    }

    handleRoomNameInput(event) {
        this.setState({roomName: event.target.value, roomNameValid: event.target.checkValidity()});
    }

    handleSwitchMode(joinState){
        this.setState({joinRoom: joinState, roomCode: "", roomName: "", roomNameValid: false});
    }

    async onCreateRoom(event){
        this.setState({loading: true});
        try {
            let response = await CreateRoom(this.state.roomName);
            console.log(response);
            if (response.status === "success"){
                this.props.history.push(`/room/${response.data.key}`)
            } else {
                throw response;
            }
        } catch (e) {
            console.error(e);
            this.setState({loading: false})
        }
    }

    render() {
        let button, input;
        if (this.state.joinRoom){
            input = <RoomCodeInput disabled={this.state.loading} onChange={this.handleRoomCodeInput} code={this.state.roomCode}/>;
            button = <JoinRoomButton loading={this.state.loading} roomCode={this.state.roomCode} onModeSwitch={this.handleSwitchMode}/>;
        } else {
            input = <RoomNameInput disabled={this.state.loading} onChange={this.handleRoomNameInput} name={this.state.roomName}/>;
            button = <CreateRoomButton onCreateRoom={this.onCreateRoom} loading={this.state.loading} valid={this.state.roomNameValid} onModeSwitch={this.handleSwitchMode}/>;
        }
        return (
            <header className="masthead d-flex">
                <Container className="text-center my-auto">
                    <div className="mx-auto" style={{width: "max-content"}}>
                        <img src={logo} alt="Project Klaus" style={{maxHeight: "240px"}}/>
                        <h1 className="mb-3" style={{fontWeight: 600, fontSize: "70px"}}>Project•Klaus</h1>
                        {input}
                        {button}
                    </div>
                </Container>
            </header>
        );
    }
}

export default Index;