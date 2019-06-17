import React from 'react';
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

export class OwnedFundingTab extends ReactiveComponent{
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
                    <FundingCards count={runtime.fundingFactory.ownedFundingCount(this.lookup)} account={this.lookup} type={2} onCardClick={onCardClick}/>
                }/>
                <CreateFunding/>
                <CreateRequest/>
            </div>
        )
    }
}

class CreateFunding extends ReactiveComponent{
    constructor(){
        super();
        this.skAccount = new Bond;
        this.projectName = new Bond;
        this.targetAmount = new Bond;
        this.supportAmount = new Bond;
        this.expiry = new Bond;
    }



    readyRender(){
        return <Segment style={{ margin: '1em' }} padded>
            <Header as='h2'>
                <Icon name='thumbs up' />
                <Header.Content>
                    Let's create a crowdfunding project
                    <Header.Subheader>Feel free to create your crowdfunding project</Header.Subheader>
                </Header.Content>
            </Header>
            <div style={{ paddingBottom: '1em' }}>
                <div style={{ fontSize: 'small' }}>Project Name</div>
                <InputBond
                    bond={this.projectName}
                    placeholder='Type your project name'
                    validator={n => n || null}
                />
            </div>
            <div style={{ paddingBottom: '1em' }}>
                <div style={{ fontSize: 'small' }}>Target Amount</div>
                <BalanceBond bond={this.targetAmount} />
            </div>
            <div style={{ paddingBottom: '1em' }}>
                <div style={{ fontSize: 'small' }}>Support Amount</div>
                <BalanceBond bond={this.supportAmount} />
            </div>
            <div style={{ paddingBottom: '1em' }}>
                <div style={{ fontSize: 'small' }}>Expiry</div>
                <InputBond
                    bond={this.expiry}
                    placeholder='Expiry Block'
                    validator={n => n || null}
                />
            </div>
            <div style={{ paddingBottom: '1em' }}>
                <div style={{ fontSize: 'small' }}>Account</div>
                <SignerBond bond={this.skAccount} />
                <If condition={this.skAccount.ready()} then={<span>
                    <Label>Balance
                        <Label.Detail>
                            <Pretty value={runtime.balances.freeBalance(this.skAccount)} />
                        </Label.Detail>
                    </Label>
                    <Label>Nonce
                        <Label.Detail>
                            <Pretty value={runtime.system.accountNonce(this.skAccount)} />
                        </Label.Detail>
                    </Label>
                </span>} />
            </div>
            <TransactButton
                content="Create Funding"
                icon='send'
                tx={{
                    sender: runtime.indices.tryIndex(this.skAccount),
                    call: calls.fundingFactory.createFunding(this.projectName.map(stringToBytes), this.targetAmount, this.supportAmount, this.expiry),
                    compact: false,
                    longevity: true
                }}
            />
        </Segment>
    }
}

class CreateRequest extends ReactiveComponent{
    constructor(){
        super();
        this.skAccount = new Bond;
        this.fundingId = new Bond;
        this.purpose = new Bond;
        this.cost = new Bond;
        this.expiry = new Bond;
    }



    readyRender(){
        return <Segment style={{ margin: '1em' }} padded>
            <Header as='h2'>
                <Icon name='send' />
                <Header.Content>
                    Create a request
                    <Header.Subheader>Create your cost request</Header.Subheader>
                </Header.Content>
            </Header>
            <div style={{ paddingBottom: '1em' }}>
                <div style={{ fontSize: 'small' }}>Funding Id</div>
                <InputBond
                    bond={this.fundingId}
                    placeholder='Type the funding id'
                    validator={id => id || null}
                />
            </div>
            <div style={{ paddingBottom: '1em' }}>
                <div style={{ fontSize: 'small' }}>Purpose</div>
                <InputBond
                    bond={this.purpose}
                    placeholder='Type your project name'
                    validator={n => n || null}
                />
            </div>
            <div style={{ paddingBottom: '1em' }}>
                <div style={{ fontSize: 'small' }}>Cost</div>
                <BalanceBond bond={this.cost} />
            </div>
            <div style={{ paddingBottom: '1em' }}>
                <div style={{ fontSize: 'small' }}>Expiry</div>
                <InputBond
                    bond={this.expiry}
                    placeholder='Expiry Block'
                    validator={n => n || null}
                />
            </div>
            <div style={{ paddingBottom: '1em' }}>
                <div style={{ fontSize: 'small' }}>Account</div>
                <SignerBond bond={this.skAccount} />
                <If condition={this.skAccount.ready()} then={<span>
                    <Label>Balance
                        <Label.Detail>
                            <Pretty value={runtime.balances.freeBalance(this.skAccount)} />
                        </Label.Detail>
                    </Label>
                    <Label>Nonce
                        <Label.Detail>
                            <Pretty value={runtime.system.accountNonce(this.skAccount)} />
                        </Label.Detail>
                    </Label>
                </span>} />
            </div>
            <TransactButton
                content="Create Request"
                icon='send'
                tx={{
                    sender: runtime.indices.tryIndex(this.skAccount),
                    call: calls.request.createRequest(this.fundingId, this.purpose.map(stringToBytes), this.cost, this.expiry),
                    compact: false,
                    longevity: true
                }}
            />
        </Segment>
    }
}