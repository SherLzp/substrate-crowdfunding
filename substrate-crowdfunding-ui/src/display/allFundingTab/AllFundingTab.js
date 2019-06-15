import React from 'react';
import { ReactiveComponent, If } from 'oo7-react';
import {FundingCards} from '../../FundingCards'
import { Icon, Accordion, List, Checkbox, Label, Header, Segment, Divider, Button } from 'semantic-ui-react';
import { calls,runtime, Hash, pretty } from 'oo7-substrate';
import { Bond } from 'oo7';
import { AccountIdBond, SignerBond } from '../../AccountIdBond.jsx';
import { InputBond } from '../../InputBond.jsx';
import { BalanceBond } from '../../BalanceBond.jsx';
import { TransactButton } from '../../TransactButton.jsx';
import { TransformBondButton } from '../../TransformBondButton';
import { Pretty } from '../../Pretty';

export class AllFundingTab extends React.Component {

    constructor() {
        super();
        this.seletedFundingDetail = '';
    }

    render() {
        const onCardClick = (selectedFundingDetail) => {
            this.seletedFundingDetail = selectedFundingDetail;
            console.log(this.seletedFundingDetail);
        };
        let type = 1;
        return (
            <div>
                <FundingCards count={runtime.fundingFactory.allFundingCount} account={null} type={type} onCardClick={onCardClick}/>
                <Invest detail={this.seletedFundingDetail}/>
            </div>
        )
    }
}

class Invest extends ReactiveComponent{
    constructor(props){
        super(['detail']);
        this.skAccount = new Bond;
        this.fundingId = new Bond;
        this.amount = new Bond;
    }

    readyRender(){
        return <Segment style={{ margin: '1em' }} padded>
            <Header as='h2'>
                <Icon name='send' />
                <Header.Content>
                    Let's invest
                    <Header.Subheader>Support your favourite funding project</Header.Subheader>
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
                <div style={{ fontSize: 'small' }}>invest amount</div>
                <BalanceBond bond={this.amount} />
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
                content="Invest"
                icon='send'
                tx={{
                    sender: runtime.indices.tryIndex(this.skAccount),
                    call: calls.fundingFactory.invest(this.fundingId, this.amount),
                    compact: false,
                    longevity: true
                }}
            />
        </Segment>
    }
}