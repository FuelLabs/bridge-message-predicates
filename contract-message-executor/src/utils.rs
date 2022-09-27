use fuels::contract::script::Script;
use fuels::prelude::*;
use fuels::tx::{
    Address, AssetId, Bytes32, Contract, Input, Output, Receipt, Transaction, TxPointer, UtxoId,
    Word,
};

const CONTRACT_MESSAGE_MIN_GAS: u64 = 1_200_000;
const CONTRACT_MESSAGE_SCRIPT_BINARY: &str =
    "../contract-message-script/out/debug/contract_message_script.bin";
const CONTRACT_MESSAGE_PREDICATE_BINARY: &str =
    "../contract-message-predicate/out/debug/contract_message_predicate.bin";

/// Gets the message to contract script
pub async fn get_contract_message_script() -> Vec<u8> {
    let script_bytecode = std::fs::read(CONTRACT_MESSAGE_SCRIPT_BINARY).unwrap();
    script_bytecode
}

/// Gets the message to contract predicate
pub async fn get_contract_message_predicate() -> (Vec<u8>, Address) {
    let predicate_bytecode = std::fs::read(CONTRACT_MESSAGE_PREDICATE_BINARY).unwrap();
    let predicate_root = Address::from(*Contract::root_from_code(&predicate_bytecode));
    (predicate_bytecode, predicate_root)
}

/// Build a message-to-contract transaction with the given input coins and outputs
/// note: unspent gas is returned to the owner of the given gas input
pub async fn build_contract_message_tx(
    message: Input,
    contract_id: ContractId,
    gas_coin: Input,
    params: TxParameters,
) -> Transaction {
    // Get the script and predicate for contract messages
    let script_bytecode = get_contract_message_script().await;

    // Start building tx list of outputs
    let mut tx_outputs: Vec<Output> = Vec::new();
    tx_outputs.push(Output::Contract {
        input_index: 0u8,
        balance_root: Bytes32::zeroed(),
        state_root: Bytes32::zeroed(),
    });

    // Build a change output for the owner of the provided gas coin input
    match gas_coin {
        Input::CoinSigned { owner, .. } | Input::CoinPredicate { owner, .. } => {
            // Add change output
            tx_outputs.push(Output::Change {
                to: owner.clone(),
                amount: 0,
                asset_id: AssetId::default(),
            });
        }
        _ => {
            // do nothing
        }
    }

    // Build variable output
    tx_outputs.push(Output::Variable {
        to: Address::default(),
        amount: Word::default(),
        asset_id: AssetId::default(),
    });

    // Start building tx list of inputs
    let mut tx_inputs: Vec<Input> = Vec::new();

    // Build contract input   TO DO: Is this zeroing correct ?
    let contract_input = Input::Contract {
        utxo_id: UtxoId::new(Bytes32::zeroed(), 0u8),
        balance_root: Bytes32::zeroed(),
        state_root: Bytes32::zeroed(),
        tx_pointer: TxPointer::default(),
        contract_id: contract_id.into(),
    };

    tx_inputs.push(contract_input);
    tx_inputs.push(message);
    tx_inputs.push(gas_coin);

    // Create the trnsaction
    Transaction::Script {
        gas_price: params.gas_price,
        gas_limit: CONTRACT_MESSAGE_MIN_GAS,
        maturity: params.maturity,
        receipts_root: Default::default(),
        script: script_bytecode,
        script_data: vec![],
        inputs: tx_inputs,
        outputs: tx_outputs,
        witnesses: vec![],
        metadata: None,
    }
}

/// Signs and broadcasts a transaction
pub async fn sign_and_call_tx(wallet: &WalletUnlocked, tx: &mut Transaction) -> Vec<Receipt> {
    // Get provider and client
    let provider = wallet.get_provider().unwrap();

    // Sign transaction and call
    wallet.sign_transaction(tx).await.unwrap();
    let script = Script::new(tx.clone());
    script.call(provider).await.unwrap()
}

/// Builds and broadcasts as transaction relaying a message
pub async fn relay_message_to_contract(wallet: &WalletUnlocked, message: Input, data: Vec<u8>) -> Vec<Receipt> {
    
    // Extract contract ID from message data (first 32 bytes)
    let contract_id_slice = &data[..32];
    let contract_id_array: [u8; 32] = contract_id_slice.try_into().unwrap(); 
    let contract_id = ContractId::from(contract_id_array);

    // TO DO: Get a coin from the wallet to pay for gas
    // Note : Need to make sure coin is big enough to cover gas
    let gas_coin = Input::CoinSigned {
        utxo_id: UtxoId::default(),
        owner: Address::default(),
        amount: 0,
        asset_id: AssetId::default(),
        tx_pointer: TxPointer::default(),
        witness_index: 0,
        maturity: 0,
    };

    // Build transaction
    let mut tx =
        build_contract_message_tx(message, contract_id, gas_coin, TxParameters::default()).await;

    // Sign transaction and call
    sign_and_call_tx(wallet, &mut tx).await
}
