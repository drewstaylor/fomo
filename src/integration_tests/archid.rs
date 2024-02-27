#![cfg(test)]
use cosmwasm_std::{
    Addr, Coin, Uint128,
};
use cw_multi_test::Executor;

use crate::integration_tests::util::{
    bank_query, create_archid, create_cw721, create_netwars, mint_native, mock_app,
    query,
};

use archid_registry::{
    state::Config as RegistryConfig, msg::ExecuteMsg as ExecuteMsgArchid
};


use crate::msg::{
    ExecuteMsg, QueryMsg,
};
use crate::contract::DENOM;
use crate::state::{State};

#[test]
fn test_enforce_archid() {
    let mut app = mock_app();
    
    // netwars owner deploys netwars
    let netwars_admin = Addr::unchecked("netwars_deployer");
    // depositor owns ARCH
    let depositor = Addr::unchecked("arch_owner");

    // mint native to netwars_admin and depositor
    mint_native(
        &mut app,
        netwars_admin.to_string(),
        Uint128::from(100000000000000000000_u128), // 100 ARCH as aarch
    );
    mint_native(
        &mut app,
        depositor.to_string(),
        Uint128::from(100000000000000000000_u128), // 100 ARCH as aarch
    );
    

    // netwars_admin deploys archid registry
    let archid_addr = create_archid(
        &mut app,
        netwars_admin.clone(),
        Addr::unchecked("empty"),
        Uint128::from(5000u64),
        86400,
    );

    // netwars_admin deploys archid token
    let cw721_addr = create_cw721(&mut app, &archid_addr);

    // netwars_admin updates registry config with archid token addr
    let update_msg = ExecuteMsgArchid::UpdateConfig {
        config: RegistryConfig {
            admin: netwars_admin.clone(),
            wallet: netwars_admin.clone(),
            cw721: cw721_addr,
            base_cost: Uint128::from(5000u64),
            base_expiration: 86400,
        },
    };
    let _registry_update = app.execute_contract(
        netwars_admin.clone(),
        archid_addr.clone(),
        &update_msg, 
        &[]
    );

    // netwars_admin deploys netwars contract with archids enabled
    let expiration: u64 = 604800; // ~1 week
    let min_deposit =  Uint128::from(1000000000000000000_u128); // 1 ARCH as aarch
    let extension_length: u64 = 3600; // 1 hour 
    let stale: u64 = 604800; // ~1 week
    let reset_length: u64 = 604800; // ~1 week
    let netwars_addr: Addr = create_netwars(
        &mut app, 
        &netwars_admin, 
        Some(archid_addr.clone()),
        expiration.clone(), 
        min_deposit.clone(),
        extension_length.clone(),
        stale,
        reset_length,
        &[],
    );

    // contract balance (netwars prize) is currently 0
    let netwars_balance: Coin = bank_query(&mut app, &netwars_addr);
    assert_eq!(netwars_balance.amount, Uint128::from(0_u128));

    // initial game state (no deposits)
    let initial_game_state: State = query(
        &mut app,
        netwars_addr.clone(),
        QueryMsg::Game{},
    ).unwrap();

    // exceuting deposit fails if sender 
    // does not own a valid ArchID
    assert!(app
        .execute_contract(
            depositor.clone(), 
            netwars_addr.clone(), 
            &ExecuteMsg::Deposit{}, 
            &[Coin {
                denom: String::from(DENOM),
                amount: Uint128::from(1000000000000000000_u128)
            }]
        ).is_err()
    );

    // depositor registers an ArchID
    let register_msg = ExecuteMsgArchid::Register {
        name: String::from("first_archid"),
    };
    let _res = app.execute_contract(
        depositor.clone(),
        archid_addr.clone(),
        &register_msg,
        &[Coin {
            denom: String::from(DENOM),
            amount: Uint128::from(5000u128),
        }],
    );

    // now depositor can make deposits
    let _res = app.execute_contract(
        depositor.clone(), 
        netwars_addr.clone(), 
        &ExecuteMsg::Deposit{}, 
        &[Coin {
            denom: String::from(DENOM),
            amount: Uint128::from(1000000000000000000_u128)
        }]
    );
    
    let game_query: State = query(
        &mut app,
        netwars_addr.clone(),
        QueryMsg::Game{},
    ).unwrap();

    // game expiration increased exactly as expected
    let expected_expiration: u64 = initial_game_state.expiration + extension_length;
    assert_eq!(game_query.expiration, expected_expiration);

    // game prize increased exactly as expected
    let netwars_balance: Coin = bank_query(&mut app, &netwars_addr);
    assert_eq!(netwars_balance.amount, Uint128::from(1000000000000000000_u128));
}