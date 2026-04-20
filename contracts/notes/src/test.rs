#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup() -> (Env, Address, GachaContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, GachaContract);
    let client = GachaContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init(&admin);
    (env, admin, client)
}

#[test]
fn test_mint_and_balance() {
    let (env, admin, client) = setup();
    let user = Address::generate(&env);
    client.mint_coins(&admin, &user, &1000u64);
    assert_eq!(client.get_balance(&user), 1000u64);
}