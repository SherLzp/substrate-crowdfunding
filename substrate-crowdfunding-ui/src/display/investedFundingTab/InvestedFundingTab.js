import React,{Component} from 'react';
import { ReactiveComponent, If } from 'oo7-react';
import {FundingCards} from '../../FundingCards'
import { Icon, Accordion, List, Checkbox, Label, Header, Segment, Divider, Button } from 'semantic-ui-react';
import { calls,runtime, BlockNumber, VecU8 } from 'oo7-substrate';
import { Bond } from 'oo7';
import { AccountIdBond, SignerBond } from '../../AccountIdBond.jsx';
import { InputBond } from '../../InputBond.jsx';
import { BalanceBond } from '../../BalanceBond.jsx';
import { TransactButton } from '../../TransactButton.jsx';
import { TransformBondButton } from '../../TransformBondButton';
import { Pretty } from '../../Pretty';
import { RequestRows } from '../common/RequestRows';

export class InvestedFundingTab extends ReactiveComponent{
    constructor(){
        super();
        this.lookup = new Bond;
        this.seletedFundingDetail = '';
    }

    readyRender(){
        const onCardClick = (selectedFundingDetail) => {
            this.seletedFundingDetail = selectedFundingDetail;
            console.log(this.seletedFundingDetail);
        };
        return(
            <div>
                <div style={{ paddingBottom: '1em' }}>
                    <div style={{ fontSize: 'small' }}>Account</div>
                    <AccountIdBond bond={this.lookup} />
                    <If condition={this.lookup.ready()} then={<span>
                        <Label>Balance
                            <Label.Detail>
                                <Pretty value={runtime.balances.freeBalance(this.lookup)} />
                            </Label.Detail>
                        </Label>
                        <Label>Nonce
                            <Label.Detail>
                                <Pretty value={runtime.system.accountNonce(this.lookup)} />
                            </Label.Detail>
                        </Label>
                    </span>}/>
                </div>
                <If condition={this.lookup.ready()} then={
                    <FundingCards count={runtime.fundingFactory.investedFundingsCount(this.lookup)} account={this.lookup} type={3} onCardClick={onCardClick}/>
                }/>
                <Request/>
            </div>
        )
    }
}

class Request extends ReactiveComponent{
    constructor(props){
        super();
        this.fundingId = new Bond;
    }

    readyRender(){
        return (<Segment style={{ margin: '1em' }} padded>
                <Header as='h2'>
                    <Icon name='send' />
                    <Header.Content>
                        All Requests
                        <Header.Subheader>Support the owner's cost</Header.Subheader>
                    </Header.Content>
                </Header>
            <div style={{ paddingBottom: '1em' }}>
                <RequestRows count={runtime.request.allRequestCount}/>
            </div>
            </Segment>)
    }
}