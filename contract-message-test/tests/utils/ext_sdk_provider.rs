/**
 * This module contains functions that should eventually
 * be made part of the fuels-rs sdk repo.
 */
use std::str::FromStr;

use fuel_crypto::Hasher;

use fuels::contract::script::Script;
use fuels::prelude::*;
use fuels::tx::{
    Address, AssetId, Bytes32, Contract as tx_contract, Input, MessageId, Output, Receipt,
    Transaction, TxPointer, UtxoId, Word,
};

const MESSAGE_SENDER_ADDRESS: &str =
    "0xca400d3e7710eee293786830755278e6d2b9278b4177b8b1a896ebd5f55c10bc";
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

/// Relays the given test message to the given test contract
pub async fn relay_test_contract_message(
    wallet: &WalletUnlocked,
    message_id: (Word, Vec<u8>),
    contract_id: ContractId,
) -> Vec<Receipt> {
    let min_gas = 1000;
    let native_asset: AssetId = Default::default();

    // Get provider and client
    let provider = wallet.get_provider().unwrap();

    // Get coins for gas
    let gas_coins = provider
        .get_spendable_coins(&wallet.address(), native_asset, min_gas)
        .await
        .unwrap();
    let gas_coins: Vec<Input> = gas_coins
        .into_iter()
        .map(|coin| Input::CoinSigned {
            utxo_id: UtxoId::from(coin.utxo_id.clone()),
            owner: Address::from(coin.owner.clone()),
            amount: coin.amount.clone().into(),
            asset_id: AssetId::from(coin.asset_id.clone()),
            tx_pointer: TxPointer::default(),
            witness_index: 0,
            maturity: 0,
        })
        .collect();

    // Message processing for this test message needs a variable output
    let output_variable = Output::Variable {
        to: Address::default(),
        amount: 0,
        asset_id: AssetId::default(),
    };

    let mut tx = build_contract_message_tx(
        message_id,
        contract_id,
        &gas_coins[..],
        &vec![output_variable],
        TxParameters::default(),
    )
    .await;
    wallet.sign_transaction(&mut tx).await.unwrap();
    let script = Script::new(tx);
    script.call(provider).await.unwrap()
}

/// Build an input contract given the message details
// TODO: [SDK] eventually just use MessageId to identify a message and query the client for the message details
pub async fn build_contract_message_input(message_id: (Word, Vec<u8>)) -> Input {
    let nonce: Word = Word::default();
    let sender = Address::from_str(MESSAGE_SENDER_ADDRESS).unwrap();
    let (predicate_bytecode, predicate_root) = get_contract_message_predicate().await;

    let mut hasher = Hasher::default();
    hasher.input(sender);
    hasher.input(predicate_root);
    hasher.input(nonce.to_be_bytes());
    hasher.input(predicate_root);
    hasher.input(message_id.0.to_be_bytes());
    hasher.input(&message_id.1);
    let hash_message_id = MessageId::from(*hasher.digest());

    Input::MessagePredicate {
        message_id: hash_message_id,
        sender,
        recipient: predicate_root,
        amount: message_id.0,
        nonce,
        owner: predicate_root,
        data: message_id.1,
        predicate: predicate_bytecode,
        predicate_data: vec![],
    }
}

/// Build a contract message relayer transaction with the given input coins and outputs
/// note: unspent gas is returned to the owner of the first given gas input
pub async fn build_contract_message_tx(
    message_id: (Word, Vec<u8>),
    contract_id: ContractId,
    gas_coins: &[Input],
    optional_outputs: &[Output],
    params: TxParameters,
) -> Transaction {
    let min_gas = 100000;

    // Get the script for contract messages
    let (script_bytecode, _) = get_contract_message_script().await;

    // Start building tx list of inputs/outputs
    let mut tx_inputs: Vec<Input> = Vec::new();
    let mut tx_outputs: Vec<Output> = Vec::new();

    // Build the contract input/outputs
    tx_inputs.push(Input::Contract {
        utxo_id: UtxoId::new(Bytes32::zeroed(), 0u8),
        balance_root: Bytes32::zeroed(),
        state_root: Bytes32::zeroed(),
        tx_pointer: TxPointer::default(),
        contract_id,
    });
    tx_outputs.push(Output::Contract {
        input_index: 0u8,
        balance_root: Bytes32::zeroed(),
        state_root: Bytes32::zeroed(),
    });

    // Build the message input
    let input_message = build_contract_message_input(message_id).await;
    tx_inputs.push(input_message);

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
    tx_outputs.append(&mut optional_outputs.to_vec());

    // Create the trnsaction
    Transaction::Script {
        gas_price: params.gas_price,
        gas_limit: min_gas,
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
