import React from "react";
import {Container} from "reactstrap/";

export default class NameList extends React.Component {
    constructor(props) {
        super(props);
    }

    render() {
        return (
            <div className="p-3" style={{height: "100vh", maxHeight:"100%"}}>
                <div className="h-100 w-100 bg-primary rounded" style={{opacity: "0.6"}}/>
            </div>
        );
    }
}

