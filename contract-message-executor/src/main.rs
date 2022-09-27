mod utils;

use fuels::prelude::*;
use fuels::tx::Input;
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
    let (predicate_bytecode, predicate_root) = utils::get_contract_message_predicate().await;
    let predicate_address = Bech32Address::from(predicate_root);

    // This will check for new messages every 10 seconds, and relay them.
    let period = time::Duration::new(10, 0);
    loop {
        relay_messages(
            provider.clone(),
            &wallet,
            &predicate_address,
            &predicate_bytecode,
        )
        .await;
        thread::sleep(period);
    }
}

async fn relay_messages(
    provider: Provider,
    wallet: &WalletUnlocked,
    predicate_address: &Bech32Address,
    predicate_bytecode: &Vec<u8>,
) {
    // Get unspent messages belonging to the predicate address
    let unspent_messages = provider.get_messages(predicate_address).await.unwrap();

    // Relay the messages
    for message in unspent_messages {
        // Convert the message to an Input::message_predicate
        let to_u8_bytes = |v: &[i32]| v.iter().flat_map(|e| e.to_ne_bytes()).collect::<Vec<_>>();

        let data = to_u8_bytes(&message.data);

        let message_id = Input::compute_message_id(
            &message.sender.clone().into(),
            &message.recipient.clone().into(),
            message.nonce.into(),
            &message.owner.clone().into(),
            message.amount.0,
            &data,
        );

        let message_input = Input::message_predicate(
            message_id,
            message.sender.into(),
            message.recipient.into(),
            message.amount.0,
            0,
            message.owner.into(),
            data.clone(),
            predicate_bytecode.clone(),
            vec![],
        );

        let _receipts = utils::relay_message_to_contract(wallet, message_input, data).await;
    }
}
