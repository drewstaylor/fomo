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

// When game is won winner must claim their prize
// within the allotted period; otherwise, anyone can 
// restart the game, prize pot of previous game carries
// over
#[test]
fn test_unlock_stale() {
    let mut app = mock_app();
    
    // netwars owner deploys netwars
    let netwars_admin = Addr::unchecked("netwars_deployer");
    // first_depositor owns ARCH
    let first_depositor = Addr::unchecked("arch_owner");
    // second_depositor owns ARCH
    let second_depositor = Addr::unchecked("second_arch_owner");
    // complete_random owns ARCH
    let complete_random = Addr::unchecked("complete_random");

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
    mint_native(
        &mut app,
        complete_random.to_string(),
        Uint128::from(1000000000000000000_u128), // 5 ARCH as aarch
    );
    
    // contract settings
    let expiration: u64 = 120; // 2 minutes
    let min_deposit =  Uint128::from(1000000000000000000_u128); // 1 ARCH as aarch
    let extension_length: u64 = 30; // 30 seconds
    let stale: u64 = 600; // 10 minutes
    let reset_length: u64 = 600; // 10 minutes

    // netwars_admin creates the netwars contract 
    let netwars_addr: Addr = create_netwars(
        &mut app, 
        &netwars_admin, 
        None,
        None,
        expiration, 
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
    
    // expiration is expired, but game not yet stale
    let current_time = get_block_time(&mut app);
    increment_block_time(&mut app, current_time + 300, 7);

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
    
    // game cannot be unlocked yet (not stale)
    assert!(
        app.execute_contract(
            first_depositor.clone(),
            netwars_addr.clone(),
            &ExecuteMsg::UnlockStale{},
            &[]
        ).is_err()
    );

    // winner does not claim prize, staleness begins
    let current_time = get_block_time(&mut app);
    increment_block_time(&mut app, current_time + 500, 10);

    // stale game can be unlocked by anyone
    let res = app.execute_contract(
        complete_random.clone(),
        netwars_addr.clone(),
        &ExecuteMsg::UnlockStale{},
        &[]
    );
    assert!(res.is_ok());

    // prize pool has carried over, game balance is still 17 ARCH
    let netwars_balance: Coin = bank_query(&mut app, &netwars_addr);
    // (seed funds + first deposit + second deposit)
    assert_eq!(netwars_balance.amount, Uint128::from(17000000000000000000_u128));

    // game was correctly restarted, and
    // round was increased
    let game_query: State = query(
        &mut app,
        netwars_addr.clone(),
        QueryMsg::Game{},
    ).unwrap();
    assert_eq!(game_query.round, 2_u64);

    // game play can resume with previous game's prize
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

    // complete_random makes a deposit
    let _res = app
        .execute_contract(
            complete_random.clone(),
            netwars_addr.clone(), 
            &ExecuteMsg::Deposit{}, 
            &[Coin {
                denom: String::from(DENOM),
                amount: Uint128::from(1000000000000000000_u128)
            }]
        )
        .unwrap();

    // depositor makes a second deposit
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
    
    let prize_balance: Coin = bank_query(&mut app, &netwars_addr);
    let winner_preclaim_balance: Coin = bank_query(&mut app, &first_depositor);

    // expire round 2 of the game, making the prize 
    // funds able to be claimed by the winner (gameover)
    let current_time = get_block_time(&mut app);
    increment_block_time(&mut app, current_time + 1000, 17);

    // first_depositor claims the prize (20 ARCH)
    // (resets / restarts game)
    let _res = app
        .execute_contract(
            first_depositor.clone(), 
            netwars_addr.clone(), 
            &ExecuteMsg::Claim{}, 
            &[]
        )
        .unwrap();
    
    // game balance is now 0
    let netwars_balance: Coin = bank_query(&mut app, &netwars_addr);
    assert_eq!(netwars_balance.amount, Uint128::from(0_u128));

    // winner's balance is now 117 ARCH
    // (seed funds + (all deposits - winner's deposits))
    let winner_balance: Coin = bank_query(&mut app, &first_depositor);
    assert_eq!(
        winner_balance.amount, 
        Uint128::from(prize_balance.amount + winner_preclaim_balance.amount)
    );

    // game was correctly restarted, and
    // round was increased
    let game_query: State = query(
        &mut app,
        netwars_addr.clone(),
        QueryMsg::Game{},
    ).unwrap();
    assert_eq!(game_query.round, 3_u64);
}