contract;

use contract_message_receiver::MessageReceiver;
use std::constants::ZERO_B256;
use std::inputs::{GTF_INPUT_MESSAGE_DATA, input_message_data_length};

storage {
    counter: u64 = 0,
    data1: ContractId = ContractId::from(ZERO_B256),
    data2: u64 = 0,
    data3: b256 = ZERO_B256,
    data4: Address = Address::from(ZERO_B256),
}

// Define verification abi
abi VerifyMessageData {
    #[storage(read)]
    fn get_test_counter() -> u64;
    #[storage(read)]
    fn get_test_data1() -> ContractId;
    #[storage(read)]
    fn get_test_data2() -> u64;
    #[storage(read)]
    fn get_test_data3() -> b256;
    #[storage(read)]
    fn get_test_data4() -> Address;
}

// Get the data of a message input
pub fn input_message_data<T>(index: u64, offset: u64) -> T {
    let data_ptr = __gtf::<raw_ptr>(index, GTF_INPUT_MESSAGE_DATA);
    data_ptr.add::<u64>(offset / 8).read::<T>()
}

// Implement the process_message function required to be a message receiver
impl MessageReceiver for Contract {
    #[storage(read, write)]
    #[payable]
    fn process_message(msg_idx: u8) {
        storage.counter = storage.counter + 1;

        // Parse the message data
        let data_length = input_message_data_length(msg_idx);
        if (data_length >= 32u16) {
            let contract_id: b256 = input_message_data(msg_idx, 0);
            storage.data1 = ContractId::from(contract_id);
        }
        if (data_length >= 32u16 + 8u16) {
            let num: u64 = input_message_data(msg_idx, 32);
            storage.data2 = num;
        }
        if (data_length >= 32u16 + 8u16 + 32u16) {
            let big_num: b256 = input_message_data(msg_idx, 32 + 8);
            storage.data3 = big_num;
        }
        if (data_length >= 32u16 + 8u16 + 32u16 + 32u16) {
            let address: b256 = input_message_data(msg_idx, 32 + 8 + 32);
            storage.data4 = Address::from(address);
        }
    }
}

// Implement simple getters for testing purposes
impl VerifyMessageData for Contract {
    #[storage(read)]
    fn get_test_counter() -> u64 {
        storage.counter
    }
    #[storage(read)]
    fn get_test_data1() -> ContractId {
        storage.data1
    }
    #[storage(read)]
    fn get_test_data2() -> u64 {
        storage.data2
    }
    #[storage(read)]
    fn get_test_data3() -> b256 {
        storage.data3
    }
    #[storage(read)]
    fn get_test_data4() -> Address {
        storage.data4
    }
}
