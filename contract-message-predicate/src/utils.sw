library utils;

use std::mem::read;
use std::contract_id::ContractId;
use std::constants::ZERO_B256;

// TODO: [std-lib] remove once standard library functions have been added
const GTF_SCRIPT_SCRIPT_LENGTH = 0x005;
const GTF_SCRIPT_SCRIPT = 0x00B;
const GTF_INPUT_COIN_AMOUNT = 0x105;
const GTF_INPUT_COIN_ASSET_ID = 0x106;
const GTF_INPUT_CONTRACT_CONTRACT_ID = 0x113;
const GTF_INPUT_MESSAGE_DATA_LENGTH = 0x11B;
const GTF_INPUT_MESSAGE_DATA = 0x11E;
const GTF_OUTPUT_CONTRACT_INPUT_INDEX = 0x205;

/// Get the hash of the script bytecode
// TODO: [std-lib] replace with 'tx_script_bytecode_hash'
pub fn tx_script_bytecode_hash() -> b256 {
    // Get the script memory details
    let mut result_buffer: b256 = ZERO_B256;
    let script_length = __gtf::<u64>(0, GTF_SCRIPT_SCRIPT_LENGTH);
    let script_ptr = __gtf::<u64>(0, GTF_SCRIPT_SCRIPT);

    // Run the hash opcode for the script in memory
    asm(hash: result_buffer, ptr: script_ptr, len: script_length) {
        s256 hash ptr len;
        hash: b256
    }
}

/// Get the length of a message input data
// TODO: [std-lib] replace with 'input_message_data_length'
pub fn input_message_data_length(index: u64) -> u64 {
    __gtf::<u64>(index, GTF_INPUT_MESSAGE_DATA_LENGTH)
}

/// Get the data of a message input
// TODO: [std-lib] replace with 'input_message_data'
pub fn input_message_data<T>(index: u64, offset: u64) -> T {
    read::<T>(__gtf::<u64>(index, GTF_INPUT_MESSAGE_DATA) + offset)
}

/// Get the ID of a contract input
// TODO: [std-lib] replace with 'input_contract_contract_id'
pub fn input_contract_contract_id(index: u64) -> ContractId {
    ~ContractId::from(__gtf::<b256>(index, GTF_INPUT_CONTRACT_CONTRACT_ID))
}

/// Get the asset ID of a coin input
// TODO: [std-lib] replace with 'input_coin_asset_id'
pub fn input_coin_asset_id(index: u64) -> ContractId {
    ~ContractId::from(__gtf::<b256>(index, GTF_INPUT_COIN_ASSET_ID))
}

/// Get the amount of a coin input
// TODO: [std-lib] replace with 'input_coin_amount'
pub fn input_coin_amount(index: u64) -> u64 {
    __gtf::<u64>(index, GTF_INPUT_COIN_AMOUNT)
}

/// Get the input index of a change output
// TODO: [std-lib] replace with 'output_contract_input_index'
pub fn output_contract_input_index(index: u64) -> u8 {
    __gtf::<u8>(index, GTF_OUTPUT_CONTRACT_INPUT_INDEX)
}
