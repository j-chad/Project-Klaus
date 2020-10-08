import React from "react";
import {Container} from "reactstrap/";

export default class NameList extends React.Component {
    constructor(props) {
        super(props);
    }

    render() {
        return (
            <div className="h-100 p-3">
                <div className="h-100 w-100 bg-dark rounded" style={{opacity:"0.6"}}></div>
            </div>
        );
    }
}

