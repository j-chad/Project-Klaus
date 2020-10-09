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
                <Container className="text-center my-0 my-md-auto">
                    <div className="mx-auto" style={{width: "max-content", maxWidth: "100%"}}>
                        <div className="d-flex align-items-center flex-column flex-lg-row">
                            <img id="logo" src={logo} alt="Project Klaus" style={{maxHeight: "150px"}} className="d-block mb-4 mb-lg-0"/>
                            <div className="text-center text-lg-left pl-0 pl-lg-3">
                                <h1 id="main-header" className="mb-3" style={{fontWeight: 600}}>Project•Klaus</h1>
                                <h2 className="mb-0 text-primary">{this.props.name}</h2>
                            </div>
                        </div>
                        <Container className="mt-5">
                            <h3 className="text-secondary">Waiting for users...</h3>
                            <div className="arrow d-md-none">
                                <span/>
                                <span/>
                                <span/>
                            </div>
                        </Container>
                    </div>
                </Container>
            </header>
        );
    }
}

