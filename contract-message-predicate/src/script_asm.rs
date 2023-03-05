use fuel_asm::{op, RegId};

const GTF_SCRIPT_INPUTS_COUNT: u16 = 0x007;
const GTF_INPUT_TYPE: u16 = 0x101;
const GTF_MSG_DATA: u16 = 0x11D;
const GTF_MSG_AMOUNT: u16 = 0x117;

const INPUT_MESSAGE_TYPE: u16 = 2;
const INSTR_PER_WORD: u16 = 2;
const BYTES_PER_WORD: u16 = 8;

// Gets the bytecode for the message-to-contract script
pub fn bytecode() -> Vec<u8> {
    //register names
    const ZERO: RegId = RegId::ZERO;
    const ONE: RegId = RegId::ONE;
    const STACK_PTR: RegId = RegId::SP;
    const INSTR_START: RegId = RegId::IS;
    const CGAS: RegId = RegId::CGAS;
    const MEMORY_START_PTR: u8 = 0x10;
    const ASSET_ID_PTR: u8 = MEMORY_START_PTR;
    const CALL_DATA_PTR: u8 = 0x11;
    const CONTRACT_ADDR_PTR: u8 = 0x12;
    const FN_SELECTOR: u8 = 0x13;
    const MSG_AMOUNT: u8 = 0x14;

    const NUM_INPUTS: u8 = 0x15;
    const INPUT_INDEX: u8 = 0x16;
    const INPUT_TYPE: u8 = 0x17;
    const EXPECTED_INPUT_TYPE: u8 = 0x18;

    //assembly instructions
    let mut script: Vec<u8> = vec![
        //extend stack for contract call data
        op::move_(MEMORY_START_PTR, STACK_PTR), //MEMORY_START_PTR = stack pointer
        op::cfei(32 + 32 + 8 + 8), //extends current call frame stack by 32+32+8+8 bytes [base asset id, contract id, param1, param2]
        //prep call parameters
        op::addi(CALL_DATA_PTR, MEMORY_START_PTR, 32), //CALL_DATA_PTR = MEMORY_START_PTR + 32bytes [memory start pointer + 32]
        op::lw(FN_SELECTOR, INSTR_START, 18 / INSTR_PER_WORD), //FN_SELECTOR = function selector at end of program [00000000 9532D7AE]
        op::sw(CALL_DATA_PTR, FN_SELECTOR, 32 / BYTES_PER_WORD), //the 8bytes at CALL_DATA_PTR + 32 = FN_SELECTOR [00000000 9532D7AE]
        //start loop through inputs
        op::gtf(NUM_INPUTS, ZERO, GTF_SCRIPT_INPUTS_COUNT), //NUM_INPUTS = the number of inputs in the script
        op::addi(EXPECTED_INPUT_TYPE, ZERO, INPUT_MESSAGE_TYPE), //EXPECTED_INPUT_TYPE = INPUT_MESSAGE_TYPE
        //INPUT_LOOP_START:
        //skip contract call if input is not a message
        op::gtf(INPUT_TYPE, INPUT_INDEX, GTF_INPUT_TYPE), //INPUT_TYPE = the type of input for input[INPUT_INDEX]
        op::jnei(INPUT_TYPE, EXPECTED_INPUT_TYPE, 14), //skips to SKIP_CALL if INPUT_TYPE does not equal EXPECTED_INPUT_TYPE
        //update call parameters
        op::gtf(MSG_AMOUNT, INPUT_INDEX, GTF_MSG_AMOUNT), //MSG_AMOUNT = amount value of message from input[INPUT_INDEX]
        op::gtf(CONTRACT_ADDR_PTR, INPUT_INDEX, GTF_MSG_DATA), //CONTRACT_ADDR_PTR = memory location of the message data from input[INPUT_INDEX]
        op::mcpi(CALL_DATA_PTR, CONTRACT_ADDR_PTR, 32), //32 bytes at CALL_DATA_PTR = the 32 bytes at CONTRACT_ADDR_PTR
        op::sw(CALL_DATA_PTR, INPUT_INDEX, (32 + 8) / BYTES_PER_WORD), //the 8bytes at CALL_DATA_PTR + 32 + 8 = INPUT_INDEX
        //make contract call
        op::call(CALL_DATA_PTR, MSG_AMOUNT, ASSET_ID_PTR, CGAS),
        //SKIP_CALL:
        //continue looping through inputs
        op::addi(INPUT_INDEX, INPUT_INDEX, 1), //INPUT_INDEX = INPUT_INDEX + 1
        op::jnei(INPUT_INDEX, NUM_INPUTS, 7), //jumps back to INPUT_LOOP_START if INPUT_INDEX does not equal NUM_INPUTS
        //end script
        op::ret(ONE),
        op::noop(),
        //referenced data (function selector)
        //00000000 9532D7AE
    ]
    .into_iter()
    .collect();

    //add referenced data
    script.append(&mut vec![0x00, 0x00, 0x00, 0x00, 0x95, 0x32, 0xD7, 0xAE]);
    script
}
