use fuel_asm::{op, RegId};
use sha2::{Digest, Sha256};

const PROCESS_MESSAGE_FUNCTION_SIGNATURE: &str = "process_message(u8)";
const GTF_MSG_DATA: u16 = 0x11D;
const GTF_MSG_AMOUNT: u16 = 0x117;

const BYTES_PER_INSTR: u16 = 4;

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
    const CALL_DATA_FN_SEL_PTR: u8 = 0x12;
    const CONTRACT_ADDR_PTR: u8 = 0x13;
    const FN_SELECTOR_PTR: u8 = 0x14;
    const MSG_AMOUNT: u8 = 0x15;

    /* The following assembly code is intended to do the following:
     *  -Call the function `process_message` on the contract with ID that matches
     *   the first 32 bytes in the message data field, while forwarding the exact
     *   amount of base asset specified in the `InputMessage` `amount` field
     */
    let mut script: Vec<u8> = vec![
        //extend stack for contract call data
        op::move_(MEMORY_START_PTR, STACK_PTR), //MEMORY_START_PTR = stack pointer
        op::cfei(32 + 32 + 8 + 8), //extends current call frame stack by 32+32+8+8 bytes [base asset id, contract id, param1, param2]
        op::addi(CALL_DATA_PTR, MEMORY_START_PTR, 32), //CALL_DATA_PTR = MEMORY_START_PTR + 32bytes [memory start pointer + 32]
        op::addi(CALL_DATA_FN_SEL_PTR, CALL_DATA_PTR, 32 + 4), //CALL_DATA_FN_SEL_PTR = CALL_DATA_PTR + 32bytes + 4bytes [call data start pointer + 32 + 4]
        //prep call parameters
        op::gtf(MSG_AMOUNT, ZERO, GTF_MSG_AMOUNT), //MSG_AMOUNT = amount value of message from input[0]
        op::gtf(CONTRACT_ADDR_PTR, ZERO, GTF_MSG_DATA), //CONTRACT_ADDR_PTR = memory location of the message data from input[0]
        op::addi(FN_SELECTOR_PTR, INSTR_START, 11 * BYTES_PER_INSTR), //FN_SELECTOR_PTR = function selector at end of program
        op::mcpi(CALL_DATA_PTR, CONTRACT_ADDR_PTR, 32), //32 bytes at CALL_DATA_PTR = the 32 bytes at CONTRACT_ADDR_PTR
        op::mcpi(CALL_DATA_FN_SEL_PTR, FN_SELECTOR_PTR, 4), //4 bytes at CALL_DATA_FN_SEL_PTR = the 4 bytes at FN_SELECTOR_PTR
        //make contract call
        op::call(CALL_DATA_PTR, MSG_AMOUNT, ASSET_ID_PTR, CGAS),
        op::ret(ZERO),
        //referenced data (function selector)
        //00000000
    ]
    .into_iter()
    .collect();

    //add referenced data (function selector)
    script.append(&mut fn_sel_hash[0..4].to_vec());
    script
}
