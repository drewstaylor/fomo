#![cfg(test)]
use cosmwasm_std::{
    Addr, Coin, Uint128,
};
use cw_multi_test::Executor;

use crate::integration_tests::util::{
    bank_query, create_netwars, increment_block_time, get_block_time, 
    mint_native, mock_app, query,
};

use crate::msg::{
    ExecuteMsg, QueryMsg,
};
use crate::contract::DENOM;
use crate::state::{State};

// When the game is won the winner can claim
// all funds from the prize pool
#[test]
fn test_claim() {
    let mut app = mock_app();
    
    // netwars owner deploys netwars
    let netwars_admin = Addr::unchecked("netwars_deployer");
    // first_depositor owns ARCH
    let first_depositor = Addr::unchecked("arch_owner");
    // second_depositor owns ARCH
    let second_depositor = Addr::unchecked("second_arch_owner");

    // mint arch to netwars_admin first_depositor and second_depositor
    mint_native(
        &mut app,
        netwars_admin.to_string(),
        Uint128::from(15000000000000000000_u128), // 15 ARCH as aarch
    );
    mint_native(
        &mut app,
        first_depositor.to_string(),
        Uint128::from(100000000000000000000_u128), // 100 ARCH as aarch
    );
    mint_native(
        &mut app,
        second_depositor.to_string(),
        Uint128::from(1000000000000000000_u128), // 1 ARCH as aarch
    );
    
    // contract settings
    let expiration: u64 = 120; // 2 minutes
    let min_deposit =  Uint128::from(1000000000000000000_u128); // 1 ARCH as aarch
    let extension_length: u64 = 30; // 30 seconds
    let stale: u64 = 604800; // ~1 week
    let reset_length: u64 = 604800; // ~1 week

    // netwars_admin creates the netwars contract 
    let netwars_addr: Addr = create_netwars(
        &mut app, 
        &netwars_admin, 
        None,
        None,
        expiration.clone(), 
        min_deposit.clone(),
        extension_length.clone(),
        stale,
        reset_length,
        &[Coin {
            denom: String::from(DENOM),
            amount: Uint128::from(15000000000000000000_u128)
        }],
    );

    // depositor makes a deposit
    let _res = app
        .execute_contract(
            first_depositor.clone(), 
            netwars_addr.clone(), 
            &ExecuteMsg::Deposit{}, 
            &[Coin {
                denom: String::from(DENOM),
                amount: Uint128::from(1000000000000000000_u128)
            }]
        )
        .unwrap();

    // second_depositor makes a deposit
    let _res = app
        .execute_contract(
            second_depositor.clone(), 
            netwars_addr.clone(), 
            &ExecuteMsg::Deposit{}, 
            &[Coin {
                denom: String::from(DENOM),
                amount: Uint128::from(1000000000000000000_u128)
            }]
        )
        .unwrap();
    
    // if expiration is expired game winner must be
    // able to claim the prize pool amount
    let current_time = get_block_time(&mut app);
    increment_block_time(&mut app, current_time + 1000, 7);

    // depositing to an expired game fails
    // (no invalid deposits)
    assert!(
        app.execute_contract(
            first_depositor.clone(), 
            netwars_addr.clone(), 
            &ExecuteMsg::Deposit{}, 
            &[Coin {
                denom: String::from(DENOM),
                amount: Uint128::from(1000000000000000000_u128)
            }]
        ).is_err()
    );
    
    // losers can't claim prize
    assert!(app
        .execute_contract(
            first_depositor.clone(), 
            netwars_addr.clone(), 
            &ExecuteMsg::Claim{}, 
            &[]
        )
        .is_err()
    );
    
    // winner can claim prize 
    // (resets / restarts game)
    let _res = app
        .execute_contract(
            second_depositor.clone(), 
            netwars_addr.clone(), 
            &ExecuteMsg::Claim{}, 
            &[]
        )
        .unwrap();
    
    // game balance is now 0
    let netwars_balance: Coin = bank_query(&mut app, &netwars_addr);
    assert_eq!(netwars_balance.amount, Uint128::from(0_u128));

    // second_depositor's balance is now 17 ARCH 
    // (seed funds + first deposit + second deposit)
    let winner_balance: Coin = bank_query(&mut app, &second_depositor);
    assert_eq!(winner_balance.amount, Uint128::from(17000000000000000000_u128));

    // game was correctly restarted, and
    // round was increased
    let game_query: State = query(
        &mut app,
        netwars_addr.clone(),
        QueryMsg::Game{},
    ).unwrap();
    assert_eq!(game_query.round, 2_u64);

    // depositing can resume
    // fund amounts >= min_deposit will succeed
    let _res = app
        .execute_contract(
            first_depositor, 
            netwars_addr.clone(), 
            &ExecuteMsg::Deposit{}, 
            &[Coin {
                denom: String::from(DENOM),
                amount: Uint128::from(2000000000000000000_u128)
            }]
        )
        .unwrap();
    let netwars_balance: Coin = bank_query(&mut app, &netwars_addr);
    assert_eq!(netwars_balance.amount, Uint128::from(2000000000000000000_u128));
}