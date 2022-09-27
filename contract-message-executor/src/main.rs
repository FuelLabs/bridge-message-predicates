mod utils;

use fuels::prelude::*;
use std::{thread, time};

// TO DO:
// Make this whole thing more professional and robust

#[tokio::main]
async fn main() {
    // Connect to provider
    let provider = Provider::connect("node-beta-1.fuel.network").await.unwrap();

    // Unlock wallet : TO DO: how to manage secrets
    let phrase = "oblige salon price punch saddle immune slogan rare snap desert retire surprise";
    let wallet = WalletUnlocked::new_from_mnemonic_phrase(phrase, Some(provider.clone())).unwrap();

    // Get predicate bytecode root address to check for messages at
    let (_, predicate_root) = utils::get_contract_message_predicate().await;
    let predicate_address = Bech32Address::from(predicate_root);

    // This will check for new messages every 10 seconds, and relay them.
    let period = time::Duration::new(10, 0);
    loop {
        relay_messages(provider.clone(), &wallet, &predicate_address).await;
        thread::sleep(period);
    }
}

async fn relay_messages(
    provider: Provider,
    wallet: &WalletUnlocked,
    predicate_address: &Bech32Address,
) {
    // Get unspent messages belonging to the predicate address
    let unspent_messages = provider.get_messages(predicate_address).await.unwrap();

    // Relay the messages
    for message in unspent_messages {
        // TO DO: `message` is a `fuels::client::schema::message::Message`, but we need a `fuels::tx::Input`
        //utils::relay_message_to_contract(wallet, message);
    }
}
