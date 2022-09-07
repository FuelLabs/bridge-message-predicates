script;

dep utils;

use contract_message_receiver::MessageReceiver;
use std::contract_id::ContractId;
use std::constants::BASE_ASSET_ID;
use utils::{input_contract_contract_id, input_message_amount};

///////////////
// CONSTANTS //
///////////////

// The input index values
const CONTRACT_INPUT_INDEX = 0u8;
const MESSAGE_INPUT_INDEX = 1u8;

////////////
// SCRIPT //
////////////

/// Script that relays a message and sends the message amount to a contract
fn main() -> bool {
    // Get contract to send message to
    let contract_id = input_contract_contract_id(CONTRACT_INPUT_INDEX);
    let message_receiver = abi(MessageReceiver, contract_id.into());
    let message_amount = input_message_amount(MESSAGE_INPUT_INDEX);

    // Execute the message
    message_receiver.process_message {
        asset_id: BASE_ASSET_ID.into(), coins: message_amount
    }
    (MESSAGE_INPUT_INDEX);
    true
}
