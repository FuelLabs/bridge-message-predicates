predicate;

dep utils;

use std::contract_id::ContractId;
use std::assert::assert;
use std::hash::sha256;
use std::{
    tx::{
        INPUT_COIN,
        INPUT_CONTRACT,
        INPUT_MESSAGE,
        OUTPUT_CHANGE,
        OUTPUT_CONTRACT,
        OUTPUT_VARIABLE,
        tx_script_bytecode,
        tx_gas_limit,
        tx_output_type,
        tx_outputs_count,
        tx_input_type,
        tx_inputs_count,
   }
};
use utils::{contract_id_from_contract_input, contract_id_from_message_input, tx_script_bytecode_hash};

/// Predicate verifying a message input is being spent according to the rules for a valid deposit
fn main() -> bool {
    ///////////////
    // CONSTANTS //
    ///////////////

    // The minimum gas limit for the transaction not to revert out-of-gas.
    const MIN_GAS = 42;

    // The hash of the script which must spend the input belonging to this predicate
    // This ensures the coins can only be spent in a call to `TokenContract.finalizeDeposit()`
    const SPENDING_SCRIPT_HASH = 0x6856bb03b84d876ee60c890be5b0602d0f7480d375917a660da3115e8e008ddb;

    ////////////////
    // CONDITIONS //
    ////////////////

    // Verify script bytecode hash matches
    assert(tx_script_bytecode_hash() == SPENDING_SCRIPT_HASH);

    // Verify gas limit is high enough
    //TODO: does gas limit include InputMessage amount? might need to just check avail balance - InputMessage amount
    assert(tx_gas_limit() >= MIN_GAS);

    // Transaction must have exactly three inputs: a Coin input (for fees), a Message, and the token Contract (in that order)
    assert(tx_inputs_count() == 3);
    assert(tx_input_type(0) == INPUT_COIN);
    let message_data_contract_id = contract_id_from_message_input(1);
    let input_contract_id = contract_id_from_contract_input(2);

    // Check contract ID from the contract input matches the one specified in the message data
    assert(input_contract_id == message_data_contract_id);

    // Transation must have exactly 3 outputs: OutputVariable, OutputContract, and OutputChange (in that order)
    assert(tx_outputs_count() == 3);
    assert(tx_output_type(0) == OUTPUT_VARIABLE);
    assert(tx_output_type(1) == OUTPUT_CONTRACT);
    assert(tx_output_type(2) == OUTPUT_CHANGE);

    true
}
