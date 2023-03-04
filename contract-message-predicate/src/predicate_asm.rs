use fuel_asm::{op, RegId};

const ZERO: RegId = RegId::ZERO;
const ONE: RegId = RegId::ONE;
const SP: RegId = RegId::SP;
const IS: RegId = RegId::IS;
const R16: u8 = 0x10;
const R17: u8 = 0x11;
const R18: u8 = 0x12;

const GTF_SCRIPT: u16 = 0x00B;
const GTF_SCRIPT_LEN: u16 = 0x005;

// Gets the bytecode for the message-to-contract predicate
pub fn bytecode() -> Vec<u8> {
    let mut predicate: Vec<u8> = vec![
        //extend stack for storing script hash
        op::move_(R18, SP), //r18 = the stack pointer
        op::cfei(32),       //extends current call frame stack by 32 bytes
        //compute script hash
        op::gtf(R16, ZERO, GTF_SCRIPT), //r16 = the script data address
        op::gtf(R17, ZERO, GTF_SCRIPT_LEN), //r17 = the script data length
        op::s256(R18, R16, R17),        //the 32bytes at r18 = the hash of the script
        //compare hash with expected
        op::addi(R16, IS, 48),   //r16 = address of 32bytes at end of program
        op::addi(R17, ZERO, 32), //r17 = 32
        op::meq(R16, R16, R18, R17), //r16 = if the 32bytes at r19 equals the 32bytes at r17
        op::jnei(R16, ONE, 10),  //skips next instruction if r16 is 0
        op::ret(ONE),
        op::lw(R16, IS, 10), //r16 = the last word at the end of the program
        op::rvrt(R16),
        //referenced data (expected script hash, error code return)
        //00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000
        //FFFFFFFF FFFF0004
    ]
    .into_iter()
    .collect();

    //add referenced data
    let mut data: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x04];
    predicate.append(&mut crate::script_hash().to_vec());
    predicate.append(&mut data);

    predicate
}
