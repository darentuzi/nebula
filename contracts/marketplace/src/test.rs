#![cfg(test)]

extern crate std;

use crate::{MarketplaceContract, MarketplaceContractClient};
use soroban_sdk::{
    testutils::Address as _,
    token::{self},
    Address, Env,
};

fn setup_test<'a>() -> (Env, MarketplaceContractClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, MarketplaceContract);
    let client: MarketplaceContractClient<'_> = MarketplaceContractClient::new(&env, &contract_id);
    (env, client)
}

fn create_token_asset<'a>(e: &Env, asset: &Address) -> token::StellarAssetClient<'a> {
    token::StellarAssetClient::new(e, &e.register_stellar_asset_contract(asset.clone()))
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn cannot_initialize_marketplace_twice() {
    let (env, client) = setup_test();
    let admin = Address::random(&env);
    let asset = Address::random(&env);
    let token = create_token_asset(&env, &asset);
    client.init(&token.address, &admin);
    client.init(&token.address, &admin);
}

#[test]
fn can_create_listing() {
    let (env, client) = setup_test();
    let admin = Address::random(&env);
    let seller = Address::random(&env);
    let token = Address::random(&env);

    let token = create_token_asset(&env, &token);
    client.init(&token.address, &admin);

    let asset = Address::random(&env);
    client.create_listing(&seller, &asset, &100);
    let listing = client.get_listing(&asset).unwrap();

    assert_eq!(&listing.listed, &true);
    assert_eq!(&listing.owner, &seller);
    assert_eq!(&listing.price, &100)
}

#[test]
fn can_create_listing_and_pause() {
    let (env, client) = setup_test();
    let admin = Address::random(&env);
    let seller = Address::random(&env);
    let token = Address::random(&env);

    let token = create_token_asset(&env, &token);
    client.init(&token.address, &admin);

    let asset = Address::random(&env);
    client.create_listing(&seller, &asset, &100);

    client.pause_listing(&seller, &asset, &100);

    let listing = client.get_listing(&asset).unwrap();

    assert_eq!(&listing.listed, &false);
    assert_eq!(&listing.owner, &seller);
    assert_eq!(&listing.price, &100)
}

#[test]
fn can_create_listing_and_sell() {
    let (env, client) = setup_test();
    let admin = Address::random(&env);
    let seller = Address::random(&env);
    let buyer = Address::random(&env);
    let token = Address::random(&env);

    let token = create_token_asset(&env, &token);
    client.init(&token.address, &admin);

    let asset = Address::random(&env);
    client.create_listing(&seller, &asset, &100);

    token.mint(&buyer, &400);
    client.buy_listing(&buyer, &asset, &100);

    let listing = client.get_listing(&asset).unwrap();

    assert_eq!(&listing.listed, &false);
    assert_eq!(&listing.owner, &buyer);
    assert_eq!(&listing.price, &100)
}

#[test]
fn can_update_a_listing() {
    let (env, client) = setup_test();
    let admin = Address::random(&env);
    let seller = Address::random(&env);
    let token = Address::random(&env);

    let token = create_token_asset(&env, &token);
    client.init(&token.address, &admin);

    let asset = Address::random(&env);
    client.create_listing(&seller, &asset, &100);

    let listing = client.get_listing(&asset).unwrap();

    assert_eq!(&listing.listed, &true);
    assert_eq!(&listing.owner, &seller);
    assert_eq!(&listing.price, &100);

    client.update_price(&seller, &asset, &100, &200);

    let listing = client.get_listing(&asset).unwrap();
    assert_eq!(&listing.listed, &true);
    assert_eq!(&listing.owner, &seller);
    assert_eq!(&listing.price, &200);

    client.pause_listing(&seller, &asset, &200);

    let listing = client.get_listing(&asset).unwrap();
    assert_eq!(&listing.listed, &false);
    assert_eq!(&listing.owner, &seller);
    assert_eq!(&listing.price, &200);

    client.unpause_listing(&seller, &asset, &200, &190);

    let listing = client.get_listing(&asset).unwrap();
    assert_eq!(&listing.listed, &true);
    assert_eq!(&listing.owner, &seller);
    assert_eq!(&listing.price, &190);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn cannot_sell_when_unlisted() {
    let (env, client) = setup_test();
    let admin = Address::random(&env);
    let seller = Address::random(&env);
    let buyer = Address::random(&env);
    let token = Address::random(&env);

    let token = create_token_asset(&env, &token);
    client.init(&token.address, &admin);

    let asset = Address::random(&env);
    client.create_listing(&seller, &asset, &100);

    client.pause_listing(&seller, &asset, &100);

    token.mint(&buyer, &400);
    client.buy_listing(&buyer, &asset, &100);
}

#[test]
fn can_remove_a_listing() {
    let (env, client) = setup_test();
    let admin = Address::random(&env);
    let seller = Address::random(&env);
    let token = Address::random(&env);

    let token = create_token_asset(&env, &token);
    client.init(&token.address, &admin);

    let asset = Address::random(&env);
    client.create_listing(&seller, &asset, &100);
    client.remove_listing(&seller, &asset, &100);

    let listing = client.get_listing(&asset);
    assert!(listing.is_none())
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn cannot_create_negative_listing() {
    let (env, client) = setup_test();
    let admin = Address::random(&env);
    let seller = Address::random(&env);
    let token = Address::random(&env);

    let token = create_token_asset(&env, &token);
    client.init(&token.address, &admin);

    let asset = Address::random(&env);
    client.create_listing(&seller, &asset, &-100);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn cannot_do_negative_update() {
    let (env, client) = setup_test();
    let admin = Address::random(&env);
    let seller = Address::random(&env);
    let token = Address::random(&env);

    let token = create_token_asset(&env, &token);
    client.init(&token.address, &admin);

    let asset = Address::random(&env);
    client.create_listing(&seller, &asset, &100);
    client.update_price(&seller, &asset, &100, &-100)
}
