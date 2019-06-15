import React from 'react';
import {If, ReactiveComponent, Rspan} from 'oo7-react';
const { Pretty } = require('../../Pretty');
import {Table, Label} from 'semantic-ui-react'
import {calls, runtime, secretStore, Tuple} from 'oo7-substrate';
import Identicon from 'polkadot-identicon';
import {TransactButton} from "../../TransactButton";
import {AccountIdBond} from "../../AccountIdBond";
import { Bond } from 'oo7';

export class RequestRows extends ReactiveComponent{
    constructor(props) {
        super(['count']);
    }
    unreadyRender() {
        return <span>No requests found yet</span>
    }
    readyRender() {
        console.log("I'm Here RequestRows");
        let requests = [];
        // All requests
        for(var i=0; i < this.state.count; i++){
            requests.push(
                <RequestWrap hash={runtime.request.allRequestArray(i)}/>
            );
        }
        return <Table celled>
            <Table.Header>
                <Table.Row>
                    <Table.HeaderCell>Owner</Table.HeaderCell>
                    <Table.HeaderCell>Purpose</Table.HeaderCell>
                    <Table.HeaderCell>Cost</Table.HeaderCell>
                    <Table.HeaderCell>Expiry</Table.HeaderCell>
                    <Table.HeaderCell>Supporter/Total</Table.HeaderCell>
                    <Table.HeaderCell>Status</Table.HeaderCell>
                    <Table.HeaderCell>Account</Table.HeaderCell>
                    <Table.HeaderCell>Vote</Table.HeaderCell>
                </Table.Row>
            </Table.Header>

            <Table.Body>
                {
                    requests
                }
            </Table.Body>
        </Table>
    }
}

class RequestWrap extends ReactiveComponent{
    constructor(props) {
        super(['hash'])
    }

    readyRender() {
        return <RequestRow
            request={runtime.request.requests(this.state.hash)}
            owner={runtime.request.requestOwner(this.state.hash)}
        />
    }
}

class RequestRow extends ReactiveComponent{
    constructor(props){
        super(['request','owner']);
        this.lookup = new Bond;
    }

    readyRender(){
        let request = this.state.request;
        let supportCount = runtime.request.supportedOfRequest(request.request_id);
        let totalCount = runtime.fundingFactory.investAccountsCount(request.funding_id);
        let purpose = new Buffer(request.purpose).toString('ascii');
        let status = request.status;
        return <Table.Row>
            <Table.Cell>
                <Rspan>
                {secretStore().find(this.state.owner).name}
                </Rspan>
                &nbsp;
                <Identicon key={this.state.owner} account={this.state.owner} size={16}/>
            </Table.Cell>
            <Table.Cell>
                {purpose}
            </Table.Cell>
            <Table.Cell>
                <Pretty value={request.cost}/>
            </Table.Cell>
            <Table.Cell>
                <Pretty value={request.expiry}/>
            </Table.Cell>
            <Table.Cell>
                <Pretty value={supportCount}/>/<Pretty value={totalCount}/>
            </Table.Cell>
            <Table.Cell>
                {status==0?"Under Voting":status==1?"Success":"Failure"}
            </Table.Cell>
            <Table.Cell>
                <div style={{ paddingBottom: '1em' }}>
                    <div style={{ fontSize: 'small' }}>Account</div>
                    <AccountIdBond bond={this.lookup} />
                    <If condition={this.lookup.ready()} then={<span>
                        <Label>Balance
                            <Label.Detail>
                                <Pretty value={runtime.balances.freeBalance(this.lookup)} />
                            </Label.Detail>
                        </Label>
                    </span>}/>
                </div>
            </Table.Cell>
            <Table.Cell>
                <If condition={status!=1} then={
                    <TransactButton
                        content="Support"
                        icon='send'
                        tx={{
                            sender: runtime.indices.tryIndex(this.lookup),
                            call: calls.request.supportRequest(request.request_id),
                            compact: false,
                            longevity: true
                        }}
                    />
                }/>
            </Table.Cell>
        </Table.Row>
    }
}