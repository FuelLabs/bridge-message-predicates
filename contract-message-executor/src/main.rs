mod utils;

use fuels::prelude::*;

#[tokio::main]
async fn main() {
    // Connect to provider
    let provider = Provider::connect("node-beta-1.fuel.network").await.unwrap();

    // Unlock wallet : TO DO: how to manage secrets
    let phrase = "oblige salon price punch saddle immune slogan rare snap desert retire surprise";
    let wallet = WalletUnlocked::new_from_mnemonic_phrase(phrase, Some(provider.clone())).unwrap();

    // Get predicate bytecode root
    let (_, predicate_root) = utils::get_contract_message_predicate().await;

    // Get unspend messages belonging to the predicate bytecode root
    let unspent_messages = provider
        .get_messages(&Bech32Address::from(predicate_root))
        .await
        .unwrap();

    // Relay the messages
    for message in unspent_messages {
        // TO DO: `message` is a `fuels::client::schema::message::Message`, but we need a `fuels::tx::Input`
        //utils::relay_message_to_contract(&wallet, message);
    }
}
