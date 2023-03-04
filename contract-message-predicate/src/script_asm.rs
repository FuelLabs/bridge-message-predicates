use fuel_asm::{op, RegId};

const ZERO: RegId = RegId::ZERO;
const ONE: RegId = RegId::ONE;
const SP: RegId = RegId::SP;
const IS: RegId = RegId::IS;
const CGAS: RegId = RegId::CGAS;
const R16: u8 = 0x10;
const R17: u8 = 0x11;
const R18: u8 = 0x12;

const GTF_MSG_DATA: u16 = 0x11D;
const GTF_MSG_AMOUNT: u16 = 0x117;

// Gets the bytecode for the message-to-contract script
pub fn bytecode() -> Vec<u8> {
    let mut script: Vec<u8> = vec![
        //extend stack for contract call data
        op::move_(R18, SP),        //r18 = the stack pointer
        op::cfei(32 + 32 + 8 + 8), //extends current call frame stack by 32+32+8+8 bytes [base asset id, contract id, param1, param2]
        //prep call parameters
        op::addi(R16, R18, 32), //r16 = r18 + 32bytes [previous stack pointer + 32]
        op::lw(R17, IS, 6),     //r17 = function selector at end of program [00000000 9532D7AE]
        op::sw(R16, R17, 4),    //the 8bytes at address r16 + 32 = r17 [00000000 9532D7AE]
        op::sw(R16, ZERO, 5),   //the 8bytes at address r16 + 32 + 8 = 0
        //prep call contract id
        op::gtf(R17, ZERO, GTF_MSG_DATA), //r17 = memory location of the message data from input 0
        op::mcpi(R16, R17, 32),           //the 32 bytes at r16 = the 32 bytes at r17
        //prep coin amount and id
        op::gtf(R17, ZERO, GTF_MSG_AMOUNT), //r17 is set to the amount value of message from input 0
        op::mcli(R18, 32),                  //the 32 bytes at r18 are set to zero
        //make contract call
        op::call(R16, R17, R18, CGAS),
        op::ret(ONE),
        //referenced data (function selector)
        //00000000 9532D7AE
    ]
    .into_iter()
    .collect();

    //add referenced data
    let mut data: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x95, 0x32, 0xD7, 0xAE];
    script.append(&mut data);
    script
}
