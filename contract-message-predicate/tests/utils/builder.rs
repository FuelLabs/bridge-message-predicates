use std::collections::HashMap;

use fuel_core_interfaces::common::prelude::Word;
use fuels::prelude::{Address, AssetId, ScriptTransaction, TxParameters};
use fuels::tx::Bytes32;
use fuels::types::coin_type::CoinType;
use fuels::types::{
    input::Input,
    output::Output,
    transaction_builders::{ScriptTransactionBuilder, TransactionBuilder},
};

/// Build a message-to-contract transaction with the given input coins and outputs
/// note: unspent gas is returned to the owner of the first given gas input
pub async fn build_contract_message_tx(
    message: Input,
    inputs: &[Input],
    outputs: &[Output],
    params: TxParameters,
) -> (ScriptTransaction, Vec<Input>, Vec<Output>) {
    // Get the script and predicate for contract messages
    let script_bytecode = fuel_contract_message_predicate::script_bytecode();

    // Start building list of inputs and outputs
    let mut tx_outputs: Vec<Output> = outputs.to_vec();
    let mut tx_inputs: Vec<Input> = vec![message];

    // Loop through inputs and add to lists
    let mut change = HashMap::new();
    for input in inputs {
        /*
        match input {
            Input::CoinSigned {
                asset_id, owner, ..
            }
            | Input::CoinPredicate {
                asset_id, owner, ..
            } => {
                change.insert(asset_id, owner);
            }
            Input::Contract { .. } => {
                tx_outputs.push(Output::Contract {
                    input_index: tx_inputs.len() as u8,
                    balance_root: Bytes32::zeroed(),
                    state_root: Bytes32::zeroed(),
                });
            }
            _ => {
                // do nothing
            }
        }
        */
        match input {
            Input::ResourceSigned { resource, .. } | Input::ResourcePredicate { resource, .. } => {
                match resource {
                    CoinType::Coin(coin) => {
                        change.insert(coin.asset_id, coin.owner.clone());
                    }
                    _ => {
                        // do nothing
                    }
                }
            }
            Input::Contract { .. } => {
                tx_outputs.push(Output::Contract {
                    input_index: tx_inputs.len() as u8,
                    balance_root: Bytes32::zeroed(),
                    state_root: Bytes32::zeroed(),
                });
            }
        }
        tx_inputs.push(input.clone());
    }
    for (asset_id, owner) in change {
        tx_outputs.push(Output::Change {
            to: owner.clone().into(),
            amount: 0,
            asset_id: asset_id.clone(),
        });
    }

    // Add variable output
    tx_outputs.push(Output::Variable {
        to: Address::default(),
        amount: Word::default(),
        asset_id: AssetId::default(),
    });

    // Create the transaction
    /*
    ScriptTransaction::new(tx_inputs, tx_outputs, params).with_script(script_bytecode)
    */
    let script_tx =
        ScriptTransactionBuilder::prepare_transfer(tx_inputs.clone(), tx_outputs.clone(), params)
            .set_script(script_bytecode)
            .build()
            .unwrap();
    (script_tx, tx_inputs, tx_outputs)
}
