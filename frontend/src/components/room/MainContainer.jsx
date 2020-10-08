import React from "react";
import {Container} from "reactstrap/";
import logo from "../../assets/images/logo.svg";

export default class MainContainer extends React.Component {
    constructor(props) {
        super(props);
    }

    render() {
        return (
            <header className="masthead d-flex">
                <Container className="text-center my-auto">
                    <div className="mx-auto" style={{width: "max-content", maxWidth: "100%"}}>
                        <div className="d-flex align-items-center">
                            <img id="logo" src={logo} alt="Project Klaus" style={{maxHeight: "150px"}}/>
                            <div style={{paddingLeft: "30px", textAlign: "left"}}>
                                <h1 id="main-header" className="mb-3" style={{fontWeight: 600}}>Project•Klaus</h1>
                                <h2 className="mb-0 text-primary">{this.props.name}</h2>
                            </div>
                        </div>
                        <Container className="mt-5">
                            <h3 className="text-secondary">Waiting for users...</h3>
                        </Container>
                    </div>
                </Container>
            </header>
        );
    }
}

