/**
 * This module contains functions that should eventually
 * be made part of the fuels-rs sdk repo.
 */
use fuel_crypto::Hasher;

use fuels::prelude::*;
use fuels::tx::{Address, AssetId, Bytes32, Contract as tx_contract, Input, Output, Transaction};

const CONTRACT_MESSAGE_MIN_GAS: u64 = 1_200_000;
const CONTRACT_MESSAGE_SCRIPT_BINARY: &str =
    "../contract-message-script/out/debug/contract_message_script.bin";
const CONTRACT_MESSAGE_PREDICATE_BINARY: &str =
    "../contract-message-predicate/out/debug/contract_message_predicate.bin";

/// Gets the message to contract script
pub async fn get_contract_message_script() -> (Vec<u8>, Bytes32) {
    let script_bytecode = std::fs::read(CONTRACT_MESSAGE_SCRIPT_BINARY).unwrap();
    let script_hash = Hasher::hash(&script_bytecode.clone());
    (script_bytecode, script_hash)
}

/// Gets the message to contract predicate
pub async fn get_contract_message_predicate() -> (Vec<u8>, Address) {
    let predicate_bytecode = std::fs::read(CONTRACT_MESSAGE_PREDICATE_BINARY).unwrap();
    let predicate_root = Address::from(*tx_contract::root_from_code(&predicate_bytecode));
    (predicate_bytecode, predicate_root)
}

/// Build a message-to-contract transaction with the given input coins and outputs
/// note: unspent gas is returned to the owner of the first given gas input
pub async fn build_contract_message_tx(
    message: Input,
    contract: Input,
    gas_coins: &[Input],
    optional_inputs: &[Input],
    optional_outputs: &[Output],
    params: TxParameters,
) -> Transaction {
    // Get the script and predicate for contract messages
    let (script_bytecode, _) = get_contract_message_script().await;

    // Start building tx list of inputs
    let mut tx_inputs: Vec<Input> = Vec::new();
    tx_inputs.push(contract);
    tx_inputs.push(message);

    // Start building tx list of outputs
    let mut tx_outputs: Vec<Output> = Vec::new();
    tx_outputs.push(Output::Contract {
        input_index: 0u8,
        balance_root: Bytes32::zeroed(),
        state_root: Bytes32::zeroed(),
    });

    // Build a change output for the owner of the first provided coin input
    if !gas_coins.is_empty() {
        let coin: &Input = &gas_coins[0];
        match coin {
            Input::CoinSigned { owner, .. } | Input::CoinPredicate { owner, .. } => {
                // Add change output
                tx_outputs.push(Output::Change {
                    to: owner.clone(),
                    amount: 0,
                    asset_id: AssetId::default(),
                });
            }
            _ => {
                // do nothing
            }
        }
    }

    // Append provided inputs and outputs
    tx_inputs.append(&mut gas_coins.to_vec());
    tx_inputs.append(&mut optional_inputs.to_vec());
    tx_outputs.append(&mut optional_outputs.to_vec());

    // Create the trnsaction
    Transaction::Script {
        gas_price: params.gas_price,
        gas_limit: CONTRACT_MESSAGE_MIN_GAS * 10,
        maturity: params.maturity,
        receipts_root: Default::default(),
        script: script_bytecode,
        script_data: vec![],
        inputs: tx_inputs,
        outputs: tx_outputs,
        witnesses: vec![],
        metadata: None,
    }
}

/*
/// Build an input message for a message-to-contract message
pub async fn build_contract_message_input(message: Message) -> Input {
    let (predicate_bytecode, _) = get_contract_message_predicate().await;
    build_input_message_predicate(message, predicate_bytecode, vec![])
}

/// Build an input message given the message details
pub fn build_input_message_predicate(message: Message, predicate: Vec<u8>, predicate_data: Vec<u8>) -> Input {
    let sender: Address = message.sender.into();
    let recipient: Address = message.recipient.into();
    let nonce: Word = message.nonce.into();
    let owner: Address = message.owner.into();
    let amount: u64 = message.amount.into();
    let data: Vec<u8> = message.data.iter().map(|d| {
        let byte: u8 = *d as u8;
        byte
    }).collect();

    let mut hasher = Hasher::default();
    hasher.input(sender);
    hasher.input(recipient);
    hasher.input(nonce.to_be_bytes());
    hasher.input(owner);
    hasher.input(amount.to_be_bytes());
    hasher.input(&data);
    let message_id = MessageId::from(*hasher.digest());

    Input::MessagePredicate {
        message_id,
        sender,
        recipient,
        amount,
        nonce,
        owner,
        data,
        predicate,
        predicate_data,
    }
}

/// Gets unspent messages sent to the given recipient
pub async fn get_messages(
    provider: &Provider,
    recipient: &Bech32Address,
) -> Result<Vec<Message>, ProviderError> {
    let mut messages: Vec<Message> = vec![];

    let mut cursor = None;

    loop {
        let pagination = PaginationRequest {
            cursor: cursor.clone(),
            results: 9999,
            direction: PageDirection::Forward,
        };
        let res = provider.client.messages(Some(&recipient.hash().to_string()), pagination).await?;

        if res.results.is_empty() {
            break;
        }
        messages.extend(res.results);
        cursor = res.cursor;
    }

    Ok(messages)
}
*/
