library utils;

use std::contract_id::ContractId;

// TODO: [std-lib] remove once standard library functions have been added
const GTF_INPUT_CONTRACT_CONTRACT_ID = 0x113;
const GTF_INPUT_MESSAGE_AMOUNT = 0x117;

/// Get the ID of a contract input
// TODO: [std-lib] replace with 'input_contract_contract_id'
pub fn input_contract_contract_id(index: u64) -> ContractId {
    ~ContractId::from(__gtf::<b256>(index, GTF_INPUT_CONTRACT_CONTRACT_ID))
}

/// Get the amount of a message input
// TODO: [std-lib] replace with 'input_message_amount'
pub fn input_message_amount(index: u64) -> u64 {
    __gtf::<u64>(index, GTF_INPUT_MESSAGE_AMOUNT)
}
