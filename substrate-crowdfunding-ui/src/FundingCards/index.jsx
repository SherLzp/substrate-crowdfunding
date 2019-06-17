import React from 'react';
import { ReactiveComponent, Rspan } from 'oo7-react';
const { Pretty } = require('../Pretty');
import { Card } from 'semantic-ui-react'
import { runtime, secretStore, Tuple} from 'oo7-substrate';
import Identicon from 'polkadot-identicon';
import { FundingAvatar } from './avatars';
import './FundingCards.css';

class FundingCard extends ReactiveComponent {
    constructor(props) {
        super(['funding', 'owner','onCardClick'])
    }

    readyRender() {
        let funding = this.state.funding;
        let amount_raised = runtime.fundingFactory.fundingSupportedAmount(funding.funding_id);
        let name = new Buffer(funding.project_name).toString('ascii');
        let status = funding.status;
        let onCardClick = this.state.onCardClick;
        let detail = {
            fundingId: '0x' + bytesToHex(funding.funding_id),
            status: funding.status
        };
        return <Card onClick={() => onCardClick && onCardClick(detail)}>
                    <FundingAvatar id={funding.funding_id} />
                    <Card.Content>
                        <Card.Header><Pretty value={funding.funding_id} className="limit-name" /></Card.Header>
                        <Card.Meta>
                            <Pretty value={funding.funding_id} className="limit-dna" />
                        </Card.Meta>
                        <Rspan>
                            <b>Owner</b>: {secretStore().find(this.state.owner).name}
                        </Rspan>
                        &nbsp;
                        <Identicon key={this.state.owner} account={this.state.owner} size={16}/>
                        <br />
                        <Rspan>
                            <b>Project Name</b>: {name}
                        </Rspan>
                        <br />
                        <Rspan>
                            <b>Target Money</b>: <Pretty value={funding.target_money}/>
                        </Rspan>
                        <br />
                        <Rspan>
                            <b>Expired Block</b>: <Pretty value={funding.expiry}/>
                        </Rspan>
                        <br />
                        <Rspan>
                            <b>Already Raised</b>: <Pretty value={amount_raised}/>
                        </Rspan>
                        <br />
                    </Card.Content>
                    <Card.Content extra>
                        <b>Status</b>: {status==0?"UnderRaising":status==1?"Success":"Failure"}
                    </Card.Content>
                </Card>;
    }
}

class FundingWrap extends ReactiveComponent {
    constructor(props) {
        super(['hash','onCardClick'])
    }

    readyRender() {
        // one level of indirection: convert a given hash
        // to the request of the actual funding data and who it belongs to
        return <FundingCard
            funding={runtime.fundingFactory.fundings(this.state.hash)}
            owner={runtime.fundingFactory.fundingOwner(this.state.hash)}
            onCardClick={this.state.onCardClick}
        />
    }
}
export class FundingCards extends ReactiveComponent {
    constructor(props) {
        super(['count','account','type','onCardClick'])
    }
    unreadyRender() {
        return <span>No funding found yet</span>
    }
    readyRender() {
        let fundings = [];
        // All fundings
        if(this.state.type === 1){
            for (var i=0; i < this.state.count; i++){
                fundings.push(
                    <div className="column" key={i}>
                        <FundingWrap hash={runtime.fundingFactory.allFundingArray(i)} onCardClick={this.state.onCardClick}/>
                    </div>
                );
            }
        }else if(this.state.type === 2){ // Owned fundings
            for(var i=0;i < this.state.count; i++){
                let tuple = new Tuple(this.state.account, i);
                fundings.push(
                    <div className="column" key={i}>
                        <FundingWrap hash={runtime.fundingFactory.ownedFundingArray(tuple)} onCardClick={this.state.onCardClick}/>
                    </div>
                );
            }
        }else if(this.state.type === 3){ // Invested fundings
            for(var i=0; i < this.state.count; i++){
                let tuple = new Tuple(this.state.account, i);
                fundings.push(
                    <div className="column" key={i}>
                        <FundingWrap hash={runtime.fundingFactory.investedFundingsArray(tuple)} onCardClick={this.state.onCardClick}/>
                    </div>
                );
            }
        }
        return <div className="ui stackable six column grid">{fundings}</div>;
    }
}
