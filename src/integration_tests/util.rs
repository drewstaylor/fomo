#![cfg(test)]
use serde::{de::DeserializeOwned, Serialize};

use cosmwasm_std::{
    Addr, BalanceResponse as BalanceResponseBank, BankQuery, Coin, Empty, from_binary, Querier, 
    QueryRequest, StdError, Timestamp, to_binary, Uint128, WasmQuery,
};
use cw_multi_test::{
    App, Contract, ContractWrapper, Executor,
};

use archid_registry::{
    msg::InstantiateMsg as InstantiateMsgArchid,
};
use archid_token::{
    InstantiateMsg as Cw721InstantiateMsg,
};

use crate::msg::InstantiateMsg;
use crate::contract::DENOM;

pub fn mock_app() -> App {
    App::default()
}

pub fn get_block_time(router: &mut App) -> u64 {
    router.block_info().time.seconds()
}

pub fn increment_block_time(router: &mut App, new_time: u64, height_incr: u64) {
    let mut curr = router.block_info();
    curr.height = curr.height + height_incr;
    curr.time = Timestamp::from_seconds(new_time);
    router.set_block(curr);
}

pub fn contract_netwars() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub fn contract_archid() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        archid_registry::contract::execute,
        archid_registry::contract::instantiate,
        archid_registry::contract::query,
    );
    Box::new(contract)
}

pub fn contract_cw721() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        archid_token::entry::execute,
        archid_token::entry::instantiate,
        archid_token::entry::query,
    );
    Box::new(contract)
}

pub fn create_netwars(
    router: &mut App, 
    owner: &Addr,
    archid_registry: Option<Addr>,
    archid_cw721: Option<Addr>,
    expiration: u64, 
    min_deposit: Uint128, 
    extensions: u64,
    stale: u64,
    reset_length: u64,
    funds: &[Coin],
) -> Addr {
    let netwars_id = router.store_code(contract_netwars());
    let msg = InstantiateMsg {
        archid_registry,
        archid_cw721,
        expiration,
        min_deposit,
        extensions,
        stale,
        reset_length,
    };
    let netwars_addr = router
        .instantiate_contract(netwars_id, owner.clone(), &msg, funds, "Netwars", None)
        .unwrap();
    
    netwars_addr
}

pub fn create_archid(
    router: &mut App,
    owner: Addr,
    cw721: Addr,
    base_cost: Uint128,
    base_expiration: u64,
) -> Addr {
    let archid_id = router.store_code(contract_archid());
    let msg = InstantiateMsgArchid {
        admin: owner.clone(),
        wallet: owner.clone(),
        cw721,
        base_cost,
        base_expiration,
    };
    let name_addr = router
        .instantiate_contract(archid_id, owner, &msg, &[], "ArchID", None)
        .unwrap();

    name_addr
}

pub fn create_cw721(router: &mut App, minter: &Addr) -> Addr {
    let cw721_id = router.store_code(contract_cw721());
    let msg = Cw721InstantiateMsg {
        name: "ArchID Token".to_string(),
        symbol: "AID".to_string(),
        minter: String::from(minter),
    };
    let contract = router
        .instantiate_contract(cw721_id, minter.clone(), &msg, &[], "cw721", None)
        .unwrap();
    contract
}

pub fn mint_native(app: &mut App, beneficiary: String, amount: Uint128) {
    app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: beneficiary,
            amount: vec![Coin {
                denom: DENOM.to_string(),
                amount: amount,
            }],
        },
    ))
    .unwrap();
}

pub fn query<M,T>(router: &mut App, target_contract: Addr, msg: M) -> Result<T, StdError>
    where
        M: Serialize + DeserializeOwned,
        T: Serialize + DeserializeOwned,
    {
        router.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: target_contract.to_string(),
            msg: to_binary(&msg).unwrap(),
        }))
    }

pub fn bank_query(app: &App, address: &Addr) -> Coin {
    let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance { 
        address: address.to_string(), 
        denom: DENOM.to_string() 
    });
    let res = app.raw_query(&to_binary(&req).unwrap()).unwrap().unwrap();
    let balance: BalanceResponseBank = from_binary(&res).unwrap();
    return balance.amount;
}