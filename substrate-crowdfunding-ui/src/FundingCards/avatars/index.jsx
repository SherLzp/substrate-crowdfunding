import React from 'react';

const IMAGES = {
    cars: [
        require('./0.png'),
        require('./1.png'),
    ],
};


function idToAttributes(id) {
    let random_num = id[10] % 2;
    return {
        car: IMAGES.cars[random_num]
    }
} 

export function FundingAvatar(props) {
    let outerStyle = {height: "132px", position: 'relative', width: '50%' },
        innerStyle = {height: "132px", position: 'relative'};

    let funding = idToAttributes(props.id);
    return <div className="">
        <div style={outerStyle}>
            <img alt='car' src={funding.car} style={innerStyle} />
        </div>
    </div>
}