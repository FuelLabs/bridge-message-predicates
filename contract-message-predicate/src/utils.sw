library utils;

use std::{
    address::Address,
    tx::{
        INPUT_COIN,
        INPUT_CONTRACT,
        INPUT_MESSAGE,
        OUTPUT_CHANGE,
        OUTPUT_CONTRACT,
        OUTPUT_VARIABLE,
        b256_from_pointer_offset,
        tx_gas_limit,
        tx_input_pointer,
        tx_input_type,
        tx_inputs_count,
        tx_output_type,
        tx_outputs_count,
    tx_script_bytecode}
};

use std::assert::assert;
use std::hash::sha256;
use std::contract_id::ContractId;

/// Get the ID of a contract input
pub fn contract_id_from_contract_input(index: u8) -> ContractId {
    // Check that input at this index is a contract input
    assert(tx_input_type(index) == INPUT_CONTRACT);
    let ptr = tx_input_pointer(index);
    let contract_id_bytes = b256_from_pointer_offset(ptr, 128); // Contract ID starts at 17th word: 16 * 8 = 128

    // TODO: Replace with actual contract id
    ~ContractId::from(0xf5dbe963c235c1e54f8732f1ecdc955df2ad8db8c9ab58eea8e1338762bf8bc2) //~ContractId::from(contract_id_bytes)
}

/// Get the contract ID from a message input's data
pub fn contract_id_from_message_input(index: u8) -> ContractId {
    // TODO: Replace with actual message check once input messages are enabled in the sdk
    assert(tx_input_type(index) == INPUT_COIN);
    ~ContractId::from(0xf5dbe963c235c1e54f8732f1ecdc955df2ad8db8c9ab58eea8e1338762bf8bc2)/*
    // Check that input at this index is a message input
    assert(tx_input_type(index) == INPUT_MESSAGE);

    let ptr = tx_input_pointer(index);
    let contract_id_bytes = b256_from_pointer_offset(ptr, 192); // Contract ID is at start of data, which is at 24th word: 24 * 8 = 192
    ~ContractId::from(contract_id_bytes)
    */
}

// From sway repo: tx.sw
use std::core::num::*;
const TX_SCRIPT_LENGTH_OFFSET = 10280;
const TX_SCRIPT_START_OFFSET = 10352;

/// Get the hash of the script bytecode
pub fn tx_script_bytecode_hash() -> b256 {
    let mut result_buffer: b256 = ~b256::min();

    asm(hash: result_buffer, script_offset: TX_SCRIPT_START_OFFSET, length_offset: TX_SCRIPT_LENGTH_OFFSET, length) {
        lw length length_offset i0; // Load the length value at the length_offset
        s256 hash script_offset length; // Hash the an array of length "length" starting from "script_offset" into "hash"
        hash: b256 // Return
    }
}
