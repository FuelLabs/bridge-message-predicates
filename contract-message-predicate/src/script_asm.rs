use fuel_asm::{op, RegId};
use sha2::{Digest, Sha256};

const PROCESS_MESSAGE_FUNCTION_SIGNATURE: &str = "process_message(u8)";
const GTF_MSG_DATA: u16 = 0x11D;
const GTF_MSG_AMOUNT: u16 = 0x117;

const INSTR_PER_WORD: u16 = 2;
const BYTES_PER_WORD: u16 = 8;

// Gets the bytecode for the message-to-contract script
pub fn bytecode() -> Vec<u8> {
    //calculate function selector
    let mut fn_sel_hasher = Sha256::new();
    fn_sel_hasher.update(PROCESS_MESSAGE_FUNCTION_SIGNATURE);
    let fn_sel_hash: [u8; 32] = fn_sel_hasher.finalize().into();

    //register names
    const ZERO: RegId = RegId::ZERO;
    const STACK_PTR: RegId = RegId::SP;
    const INSTR_START: RegId = RegId::IS;
    const CGAS: RegId = RegId::CGAS;
    const MEMORY_START_PTR: u8 = 0x10;
    const ASSET_ID_PTR: u8 = MEMORY_START_PTR;
    const CALL_DATA_PTR: u8 = 0x11;
    const CONTRACT_ADDR_PTR: u8 = 0x12;
    const FN_SELECTOR: u8 = 0x13;
    const MSG_AMOUNT: u8 = 0x14;

    /* The following assembly code is intended to do the following:
     *  -Call the function `process_message` on the contract with ID that matches
     *   the first 32 bytes in the message data field, while forwarding the exact
     *   amount of base asset specified in the `InputMessage` `amount` field
     */
    let mut script: Vec<u8> = vec![
        //extend stack for contract call data
        op::move_(MEMORY_START_PTR, STACK_PTR), //MEMORY_START_PTR = stack pointer
        op::cfei(32 + 32 + 8 + 8), //extends current call frame stack by 32+32+8+8 bytes [base asset id, contract id, param1, param2]
        //prep call parameters
        op::addi(CALL_DATA_PTR, MEMORY_START_PTR, 32), //CALL_DATA_PTR = MEMORY_START_PTR + 32bytes [memory start pointer + 32]
        op::lw(FN_SELECTOR, INSTR_START, 10 / INSTR_PER_WORD), //FN_SELECTOR = function selector at end of program [00000000 9532D7AE]
        op::gtf(MSG_AMOUNT, ZERO, GTF_MSG_AMOUNT), //MSG_AMOUNT = amount value of message from input[0]
        op::gtf(CONTRACT_ADDR_PTR, ZERO, GTF_MSG_DATA), //CONTRACT_ADDR_PTR = memory location of the message data from input[0]
        op::mcpi(CALL_DATA_PTR, CONTRACT_ADDR_PTR, 32), //32 bytes at CALL_DATA_PTR = the 32 bytes at CONTRACT_ADDR_PTR
        op::sw(CALL_DATA_PTR, FN_SELECTOR, 32 / BYTES_PER_WORD), //the 8bytes at CALL_DATA_PTR + 32 = FN_SELECTOR [00000000 9532D7AE]
        //make contract call
        op::call(CALL_DATA_PTR, MSG_AMOUNT, ASSET_ID_PTR, CGAS),
        op::ret(ZERO),
        //word aligned referenced data (function selector)
        //00000000 00000000
    ]
    .into_iter()
    .collect();

    //add referenced data (function selector)
    script.append(&mut vec![0x00, 0x00, 0x00, 0x00]);
    script.append(&mut fn_sel_hash[0..4].to_vec());
    script
}
