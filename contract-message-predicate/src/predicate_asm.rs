use fuel_asm::{op, RegId};

const GTF_SCRIPT: u16 = 0x00B;
const GTF_SCRIPT_LEN: u16 = 0x005;

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
    const ERR_CODE: u8 = 0x15;
    const VAL_32: u8 = 0x16;

    //assembly instructions
    let mut predicate: Vec<u8> = vec![
        //extend stack for storing script hash
        op::move_(SCRIPT_HASH_PTR, STACK_PTR), //SCRIPT_HASH_PTR = stack pointer
        op::cfei(32),                          //extends current call frame stack by 32 bytes
        //compute script hash
        op::gtf(SCRIPT_PTR, ZERO, GTF_SCRIPT), //SCRIPT_PTR = script data address
        op::gtf(SCRIPT_LEN, ZERO, GTF_SCRIPT_LEN), //SCRIPT_LEN = script data length
        op::s256(SCRIPT_HASH_PTR, SCRIPT_PTR, SCRIPT_LEN), //32bytes at SCRIPT_HASH_PTR = hash of the script
        //compare hash with expected
        op::addi(EXPECTED_HASH_PTR, INSTR_START, 48), //EXPECTED_HASH_PTR = address of reference data at end of program
        op::addi(VAL_32, ZERO, 32),                   //VAL_32 = 32
        op::meq(COMPARE_RESULT, EXPECTED_HASH_PTR, SCRIPT_HASH_PTR, VAL_32), //COMPARE_RESULT = if the 32bytes at SCRIPT_HASH_PTR equals the 32bytes at EXPECTED_HASH_PTR
        op::jnei(COMPARE_RESULT, ONE, 10), //skips next instruction if COMPARE_RESULT is 0
        op::ret(ONE),
        op::lw(ERR_CODE, INSTR_START, 10), //ERR_CODE = last word of reference at the end of the program
        op::rvrt(ERR_CODE),
        //referenced data (expected script hash, error code return)
        //00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        //FFFFFFFF FFFF0004
    ]
    .into_iter()
    .collect();

    //add referenced data
    predicate.append(&mut crate::script_hash().to_vec());
    predicate.append(&mut vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x04]);

    predicate
}
