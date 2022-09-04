use crate::ext_fuel_core;
use crate::ext_sdk_provider;

use std::mem::size_of;
use std::str::FromStr;

use fuels::contract::script::Script;
use fuels::prelude::*;
use fuels::signers::fuel_crypto::SecretKey;
use fuels::test_helpers::Config;
use fuels::tx::Output;
use fuels::tx::Receipt;
use fuels::tx::{Address, AssetId, Bytes32, Input, TxPointer, UtxoId, Word};

abigen!(
    TestContract,
    "../contract-message-test/out/debug/contract_message_test-flat-abi.json"
);

const MESSAGE_SENDER_ADDRESS: &str =
    "0xca400d3e7710eee293786830755278e6d2b9278b4177b8b1a896ebd5f55c10bc";
const TEST_RECEIVER_CONTRACT_BINARY: &str =
    "../contract-message-test/out/debug/contract_message_test.bin";

/// Sets up a test fuel environment with a funded wallet
pub async fn setup_environment(
    coins: Vec<(Word, AssetId)>,
    messages: Vec<(Word, Vec<u8>)>,
) -> (WalletUnlocked, TestContract, Input, Vec<Input>, Vec<Input>) {
    // Create secret for wallet
    const SIZE_SECRET_KEY: usize = size_of::<SecretKey>();
    const PADDING_BYTES: usize = SIZE_SECRET_KEY - size_of::<u64>();
    let mut secret_key: [u8; SIZE_SECRET_KEY] = [0; SIZE_SECRET_KEY];
    secret_key[PADDING_BYTES..].copy_from_slice(&(8320147306839812359u64).to_be_bytes());

    // Generate wallet
    let mut wallet = WalletUnlocked::new_from_private_key(
        SecretKey::try_from(secret_key.as_slice())
            .expect("This should never happen as we provide a [u8; SIZE_SECRET_KEY] array"),
        None,
    );

    // Generate coins for wallet
    let asset_configs: Vec<AssetConfig> = coins
        .iter()
        .map(|coin| AssetConfig {
            id: coin.1,
            num_coins: 1,
            coin_amount: coin.0,
        })
        .collect();
    let all_coins = setup_custom_assets_coins(wallet.address(), &asset_configs[..]);

    // Generate messages
    let message_nonce: Word = Word::default();
    let message_sender = Address::from_str(MESSAGE_SENDER_ADDRESS).unwrap();
    let (predicate_bytecode, predicate_root) =
        ext_sdk_provider::get_contract_message_predicate().await;
    let all_messages = messages
        .iter()
        .map(|message| {
            ext_fuel_core::setup_single_message(
                message_sender,
                predicate_root,
                message.0,
                message_nonce,
                message.1.clone(),
            )
        })
        .collect();

    // Create the client and provider
    let mut provider_config = Config::local_node();
    provider_config.predicates = true;
    let (client, _) = ext_fuel_core::setup_test_client_with_messages(
        &all_coins,
        &all_messages,
        Some(provider_config),
        None,
    )
    .await;
    let provider = Provider::new(client);

    // Add provider to wallet
    wallet.set_provider(provider.clone());

    // Deploy the target contract used for testing processing messages
    let test_contract_id = Contract::deploy(
        TEST_RECEIVER_CONTRACT_BINARY,
        &wallet,
        TxParameters::default(),
        StorageConfiguration::default(),
    )
    .await
    .unwrap();
    let test_contract =
        TestContractBuilder::new(test_contract_id.to_string(), wallet.clone()).build();

    // Build inputs for provided coins
    let coin_inputs: Vec<Input> = all_coins
        .into_iter()
        .map(|coin| Input::CoinSigned {
            utxo_id: UtxoId::from(coin.0.clone()),
            owner: Address::from(coin.1.owner.clone()),
            amount: coin.1.amount.clone().into(),
            asset_id: AssetId::from(coin.1.asset_id.clone()),
            tx_pointer: TxPointer::default(),
            witness_index: 0,
            maturity: 0,
        })
        .collect();

    // Build inputs for provided messages
    let message_inputs: Vec<Input> = all_messages
        .iter()
        .map(|message| Input::MessagePredicate {
            message_id: message.id(),
            sender: Address::from(message.sender.clone()),
            recipient: Address::from(message.recipient.clone()),
            amount: message.amount,
            nonce: message.nonce,
            owner: Address::from(message.owner.clone()),
            data: message.data.clone(),
            predicate: predicate_bytecode.clone(),
            predicate_data: vec![],
        })
        .collect();

    // Build contract input
    let contract_input = Input::Contract {
        utxo_id: UtxoId::new(Bytes32::zeroed(), 0u8),
        balance_root: Bytes32::zeroed(),
        state_root: Bytes32::zeroed(),
        tx_pointer: TxPointer::default(),
        contract_id: test_contract_id.into(),
    };

    (
        wallet,
        test_contract,
        contract_input,
        coin_inputs,
        message_inputs,
    )
}

/// Relays a message-to-contract message
pub async fn relay_message_to_contract(
    wallet: &WalletUnlocked,
    message: Input,
    contract: Input,
    gas_coins: &[Input],
    optional_outputs: &[Output],
) -> Vec<Receipt> {
    // Get provider and client
    let provider = wallet.get_provider().unwrap();

    // Build transaction
    let mut tx = ext_sdk_provider::build_contract_message_tx(
        message,
        contract,
        gas_coins,
        optional_outputs,
        TxParameters::default(),
    )
    .await;

    // Sign transaction and call
    wallet.sign_transaction(&mut tx).await.unwrap();
    let script = Script::new(tx);
    script.call(provider).await.unwrap()
}

/// Prefixes the given bytes with the test contract ID
pub async fn prefix_contract_id(data: Vec<u8>) -> Vec<u8> {
    // Compute the test contract ID
    let storage_configuration = StorageConfiguration::default();
    let compiled_contract = Contract::load_sway_contract(
        TEST_RECEIVER_CONTRACT_BINARY,
        &storage_configuration.storage_path,
    )
    .unwrap();
    let (test_contract_id, _) = Contract::compute_contract_id_and_state_root(&compiled_contract);

    // Turn contract id into array with the given data appended to it
    let test_contract_id: [u8; 32] = test_contract_id.into();
    let mut test_contract_id = test_contract_id.to_vec();
    test_contract_id.append(&mut data.clone());
    test_contract_id
}

/*
/// Build a contract message relayer transaction with the given input coins and outputs
/// note: unspent gas is returned to the owner of the first given gas input
pub async fn build_contract_message_tx(
    message_id: (Word, Vec<u8>),
    contract_id: ContractId,
    gas_coins: &[Input],
    optional_outputs: &[Output],
    params: TxParameters
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
            Input::CoinSigned{owner, ..} | Input::CoinPredicate{owner, ..} => {
                // Add change output
                tx_outputs.push(Output::Change {
                    to: owner.clone(),
                    amount: 0,
                    asset_id: AssetId::default(),
                });
            },
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
*/
