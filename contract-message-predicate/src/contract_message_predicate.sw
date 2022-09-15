predicate;

dep utils;

use std::{
    assert::assert,
    constants::BASE_ASSET_ID,
    contract_id::ContractId,
    inputs::{
        Input,
        input_count,
        input_type,
    },
    outputs::{
        Output,
        output_count,
        output_type,
    },
    tx::tx_gas_limit,
};
use utils::{
    input_coin_amount,
    input_coin_asset_id,
    input_contract_contract_id,
    input_message_data,
    input_message_data_length,
    output_contract_input_index,
    tx_script_bytecode_hash,
};

///////////////
// CONSTANTS //
///////////////
// The minimum gas limit for the transaction not to revert out-of-gas
// TODO: research what gas amount is reasonable
const MIN_GAS = 1_200_000;

// The hash of the script which must spend the input belonging to this predicate
const SPENDING_SCRIPT_HASH = 0x1f820272c1191516cb7477d3cd1023e9768096f37f5faba79efb0adc7c764f1c;

///////////
// UTILS //
///////////
/// Get the contract ID in the data of a message input
fn input_message_contract_id(index: u64) -> ContractId {
    // Length should be at least 32 bytes for the contract ID
    let msg_data_len = input_message_data_length(index);
    assert(msg_data_len >= 32);

    // Parse the contract ID from the message data
    let contract_id: b256 = input_message_data(index, 0);
    ~ContractId::from(contract_id)
}

/// Verifies an input at the given index is of a specific type
fn verify_input(index: u8, expected_type: Input) -> bool {
    if let expected_type = input_type(index) {
        true
    } else {
        false
    }
}

/// Verifies an output at the given index is of a specific type
fn verify_output(index: u8, expected_type: Output) -> bool {
    if let expected_type = output_type(index) {
        true
    } else {
        false
    }
}

/// Verifies the input at the given index meets expectations (returns amount of base asset coins)
fn verify_other_input(index: u8, num_inputs: u8) -> u64 {
    let mut num_coins = 0;
    if (index < num_inputs) {
        match input_type(index) {
            // Coin inputs must be of the base asset
            Input::Coin => {
                assert(input_coin_asset_id(index) == BASE_ASSET_ID);
                num_coins = input_coin_amount(index);
            },
            // Additional contract inputs are allowed
            Input::Contract => {},
            // No other input types are allowed
            _ => {
                assert(false);
            }
        }
    }
    num_coins
}

/// Verifies the output at the given index meets expectations
fn verify_other_output(index: u8, num_outputs: u8) {
    if (index < num_outputs) {
        match output_type(index) {
            // Additional contract outputs are allowed
            Output::Contract => {},
            // Any variable outputs are allowed
            Output::Variable => {},
            // Any message outputs are allowed
            Output::Message => {},
            // No other output types are allowed
            _ => {
                assert(false);
            }
        }
    }
}

///////////////
// PREDICATE //
///////////////
/// Predicate verifying a message input is being spent according to the rules for a valid message data relay to contract
fn main() -> bool {
    // Verify script bytecode hash matches
    assert(tx_script_bytecode_hash() == SPENDING_SCRIPT_HASH);

    // Verify the transaction inputs
    let mut coin_input_total: u64 = 0;
    let num_inputs = input_count();
    assert(num_inputs >= 2 && num_inputs <= 8);
    assert(verify_input(0, Input::Contract));
    assert(verify_input(1, Input::Message));
    assert(input_contract_contract_id(0) == input_message_contract_id(1));
    coin_input_total += verify_other_input(2, num_inputs);
    coin_input_total += verify_other_input(3, num_inputs);
    coin_input_total += verify_other_input(4, num_inputs);
    coin_input_total += verify_other_input(5, num_inputs);
    coin_input_total += verify_other_input(6, num_inputs);
    coin_input_total += verify_other_input(7, num_inputs);

    // Verify the transaction outputs
    // note: the OutputChange at index 1 is guaranteed to be for the base asset
    // since no other OutputChange are allowed and tx wouldn't validate if otherwise
    let num_outputs = output_count();
    assert(num_outputs >= 2 && num_outputs <= 8);
    assert(verify_output(0, Output::Contract));
    assert(verify_output(1, Output::Change));
    assert(output_contract_input_index(0) == 0);
    verify_other_output(2, num_outputs);
    verify_other_output(3, num_outputs);
    verify_other_output(4, num_outputs);
    verify_other_output(5, num_outputs);
    verify_other_output(6, num_outputs);
    verify_other_output(7, num_outputs);

    // Verify there is a minimum amount of gas to process message
    assert(tx_gas_limit() >= MIN_GAS);
    assert(coin_input_total >= MIN_GAS);

    // All checks have passed
    true
}
