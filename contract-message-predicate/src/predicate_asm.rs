use fuel_asm::{op, RegId};

const GTF_SCRIPT: u16 = 0x00B;
const GTF_SCRIPT_LEN: u16 = 0x005;
const GTF_SCRIPT_INPUTS_COUNT: u16 = 0x007;
const GTF_INPUT_TYPE: u16 = 0x101;
const GTF_MSG_DATA_LEN: u16 = 0x11A;

const INPUT_MESSAGE_TYPE: u16 = 2;
const BYTES_PER_INSTR: u16 = 4;

// Gets the bytecode for the message-to-contract predicate
pub fn bytecode() -> Vec<u8> {
    //register names
    const ZERO: RegId = RegId::ZERO;
    const ONE: RegId = RegId::ONE;
    const STACK_PTR: RegId = RegId::SP;
    const INSTR_START: RegId = RegId::IS;
    const SCRIPT_HASH_PTR: u8 = 0x10;
    const SCRIPT_PTR: u8 = 0x11;
    const SCRIPT_LEN: u8 = 0x12;
    const EXPECTED_HASH_PTR: u8 = 0x13;
    const COMPARE_RESULT: u8 = 0x14;
    const VAL_32: u8 = 0x16;
    const INPUT_INDEX: u8 = 0x17;
    const INPUT_TYPE: u8 = 0x18;
    const INPUT_MSG_DATA_LEN: u8 = 0x19;
    const EXPECTED_INPUT_TYPE: u8 = 0x20;

    /* The following assembly code is intended to do the following:
     *  -Verify that the script bytecode hash for the transaction matches that of
     *   the expected Message to Contract script
     *  -Verify there are no other `InputMessages` with data in the transaction
     *   other than the first input
     *
     * If these conditions are met, then the predicate evaluates as true.
     */
    let mut predicate: Vec<u8> = vec![
        //extend stack for storing script hash
        op::move_(SCRIPT_HASH_PTR, STACK_PTR), //SCRIPT_HASH_PTR = stack pointer
        op::cfei(32),                          //extends current call frame stack by 32 bytes
        //compute script hash
        op::gtf(SCRIPT_PTR, ZERO, GTF_SCRIPT), //SCRIPT_PTR = script data address
        op::gtf(SCRIPT_LEN, ZERO, GTF_SCRIPT_LEN), //SCRIPT_LEN = script data length
        op::s256(SCRIPT_HASH_PTR, SCRIPT_PTR, SCRIPT_LEN), //32bytes at SCRIPT_HASH_PTR = hash of the script
        //compare hash with expected
        op::addi(EXPECTED_HASH_PTR, INSTR_START, 19 * BYTES_PER_INSTR), //EXPECTED_HASH_PTR = address of reference data at end of program
        op::movi(VAL_32, 32),                                           //VAL_32 = 32
        op::meq(COMPARE_RESULT, EXPECTED_HASH_PTR, SCRIPT_HASH_PTR, VAL_32), //COMPARE_RESULT = if the 32bytes at SCRIPT_HASH_PTR equals the 32bytes at EXPECTED_HASH_PTR
        op::jnei(COMPARE_RESULT, ONE, 18), //jumps to PREDICATE_FAILURE if COMPARE_RESULT is not 1
        //confirm that no other messages with data are included
        op::gtf(INPUT_INDEX, ZERO, GTF_SCRIPT_INPUTS_COUNT), //INPUT_INDEX = the number of inputs in the script
        op::addi(EXPECTED_INPUT_TYPE, ZERO, INPUT_MESSAGE_TYPE), //EXPECTED_INPUT_TYPE = INPUT_MESSAGE_TYPE
        //LOOP_START:
        op::subi(INPUT_INDEX, INPUT_INDEX, 1), //INPUT_INDEX = INPUT_INDEX - 1
        //check if the input is a message input
        op::gtf(INPUT_TYPE, INPUT_INDEX, GTF_INPUT_TYPE), //INPUT_TYPE = the type of input for input[INPUT_INDEX]
        op::jnei(INPUT_TYPE, EXPECTED_INPUT_TYPE, 16), //skips to SKIP_DATA_CHECK if INPUT_TYPE does not equal EXPECTED_INPUT_TYPE
        //check it the input message has data
        op::gtf(INPUT_MSG_DATA_LEN, INPUT_INDEX, GTF_MSG_DATA_LEN), //INPUT_MSG_DATA_LEN = the data length of input[INPUT_INDEX]
        op::jnei(INPUT_MSG_DATA_LEN, ZERO, 18), //jumps to PREDICATE_FAILURE if INPUT_MSG_DATA_LEN does not equal 0
        //SKIP_DATA_CHECK:
        op::jnei(INPUT_INDEX, ONE, 11), //jumps back to LOOP_START if INPUT_INDEX does not equal 1
        op::ret(ONE),
        //PREDICATE_FAILURE:
        op::ret(ZERO),
        //referenced data (expected script hash)
        //00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
    ]
    .into_iter()
    .collect();

    //add referenced data (expected script hash)
    predicate.append(&mut crate::script_hash().to_vec());
    predicate
}
