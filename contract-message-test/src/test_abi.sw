library test_abi;

use std::address::Address;
use std::contract_id::ContractId;

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
