import React from 'react'
import {Tab} from 'semantic-ui-react'
import {AllFundingTab} from "./allFundingTab/AllFundingTab";
import {OwnedFundingTab} from "./ownedFundingTab/OwnedFundingTab";
import {InvestedFundingTab} from "./investedFundingTab/InvestedFundingTab";
import {ReactiveComponent} from "oo7-react";

export class TabCenter extends ReactiveComponent {
    constructor() {
        super()
        this.panes = [
            {menuItem: 'All', render: () => <Tab.Pane><AllFundingTab/></Tab.Pane>},
            {menuItem: 'Owned', render: () => <Tab.Pane><OwnedFundingTab/></Tab.Pane>},
            {menuItem: 'Invested', render: () => <Tab.Pane><InvestedFundingTab/></Tab.Pane>},
        ]
    }

    readyRender() {
        return <Tab panes={this.panes}/>
    }
}