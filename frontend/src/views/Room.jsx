import React from "react";
import {Col, Container, Row} from "reactstrap";
import {JoinRoom} from "../api/rooms";
import MainContainer from "../components/room/MainContainer";
import NameList from "../components/room/NameList";
import "../assets/scss/views/room.scss";


class Room extends React.Component {
    constructor(props) {
        super(props);

        this.state = {
            loading: false,
            name: "Loading..."
        }

        this.key = this.props.match.params.id
    }

    async componentDidMount() {
        this.setState({"loading": true});
        try {
            let response = await JoinRoom(this.key);
            if (response.data.exists){
                this.setState({
                    "name": response.data.room.name,
                    "loading": false
                });
            } else {
                this.props.history.push("/");
            }
        } catch (e) {
            console.error(e);
            this.setState({loading: false});
        }
    }

    render() {

        return (
            <Container fluid id={"room-view"} style={{overflow: "auto"}}>
                <Row style={{height: "100vh"}}>
                    <Col sm="12" md="6" lg="5" xl="3" className="px-0 order-last order-md-first">
                        <NameList/>
                    </Col>
                    <Col sm="12" md="6" lg="7" xl="9" className="px-0 order-first order-md-last">
                        <MainContainer name={this.state.name}/>
                    </Col>
                </Row>
            </Container>
        );
    }
}

export default Room;