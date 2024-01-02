#![cfg(test)]
use serde::{de::DeserializeOwned, Serialize};

use cosmwasm_std::{
    Addr, BalanceResponse as BalanceResponseBank, BankQuery, Coin, Empty, from_binary, Querier, 
    QueryRequest, StdError, Timestamp, to_binary, Uint128, WasmQuery,
};
use cw_multi_test::{
    App, Contract, ContractWrapper, Executor,
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

pub fn contract_fomo() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

pub fn create_fomo(
    router: &mut App, 
    owner: &Addr,
    expiration: u64, 
    min_deposit: Uint128, 
    extensions: u64,
    reset_length: u64,
    funds: &[Coin],
) -> Addr {
    let fomo_id = router.store_code(contract_fomo());
    let msg = InstantiateMsg {
        expiration,
        min_deposit,
        extensions,
        reset_length,
    };
    let fomo_addr = router
        .instantiate_contract(fomo_id, owner.clone(), &msg, funds, "Fomo", None)
        .unwrap();
    
    fomo_addr
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