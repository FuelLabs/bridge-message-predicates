mod utils {
    pub mod environment;
    pub mod ext_fuel_core;
    pub mod ext_sdk_provider;
}
use utils::ext_fuel_core;
use utils::ext_sdk_provider;

pub const RANDOM_SALT: &str = "0x1a896ebd5f55c10bc830755278e6d2b9278b4177b8bca400d3e7710eee293786";
pub const RANDOM_SALT2: &str = "0xd5f55c10bc830755278e6d2b9278b4177b8bca401a896eb0d3e7710eee293786";

// Test that input messages can be relayed to a contract
// and that the contract can successfully parse the message data
mod success {
    use std::str::FromStr;

    use crate::utils::environment as env;
    use fuels::prelude::Contract;
    use fuels::prelude::Salt;
    use fuels::prelude::StorageConfiguration;
    use fuels::prelude::TxParameters;
    use fuels::test_helpers::DEFAULT_COIN_AMOUNT;
    use fuels::tx::{
        Address, AssetId, Bytes32, ContractId, Input, Output, TxPointer, UtxoId, Word,
    };

    pub const RANDOM_SALT: &str =
        "0x1a896ebd5f55c10bc830755278e6d2b9278b4177b8bca400d3e7710eee293786";
    pub const RANDOM_SALT2: &str =
        "0xd5f55c10bc830755278e6d2b9278b4177b8bca401a896eb0d3e7710eee293786";

    #[tokio::test]
    async fn relay_message_with_predicate_and_script_constraint() {
        let data_word = 54321u64;
        let data_bytes = Bytes32::from_str(RANDOM_SALT).unwrap();
        let data_address = Address::from_str(RANDOM_SALT2).unwrap();
        let mut message_data = data_word.to_be_bytes().to_vec();
        message_data.append(&mut env::decode_hex(RANDOM_SALT));
        message_data.append(&mut env::decode_hex(RANDOM_SALT2));
        let message_data = env::prefix_contract_id(message_data).await;
        let message = (100, message_data);
        let coin = (DEFAULT_COIN_AMOUNT, AssetId::default());

        // Set up the environment
        let (wallet, test_contract, contract_input, coin_inputs, message_inputs) =
            env::setup_environment(vec![coin], vec![message]).await;

        // Relay the test message to the test contract
        let _receipts = env::relay_message_to_contract(
            &wallet,
            message_inputs[0].clone(),
            contract_input,
            &coin_inputs[..],
            &vec![],
            &vec![],
        )
        .await;

        // Verify test contract received the message
        let test_contract_counter = test_contract.get_test_counter().call().await.unwrap().value;
        assert_eq!(test_contract_counter, 1);

        // Verify test contract received the correct data
        let test_contract_id: ContractId = test_contract._get_contract_id().into();
        let test_contract_data1 = test_contract.get_test_data1().call().await.unwrap().value;
        assert_eq!(test_contract_data1, test_contract_id);
        let test_contract_data2 = test_contract.get_test_data2().call().await.unwrap().value;
        assert_eq!(test_contract_data2, data_word);
        let test_contract_data3 = test_contract.get_test_data3().call().await.unwrap().value;
        assert_eq!(test_contract_data3, data_bytes.to_vec()[..]);
        let test_contract_data4 = test_contract.get_test_data4().call().await.unwrap().value;
        assert_eq!(test_contract_data4, data_address);

        // Verify the message value was received by the test contract
        let provider = wallet.get_provider().unwrap();
        let test_contract_balance = provider
            .get_contract_asset_balance(test_contract._get_contract_id(), AssetId::default())
            .await
            .unwrap();
        assert_eq!(test_contract_balance, 100);
    }

    #[tokio::test]
    async fn relay_message_with_multiple_tokens_for_gas() {
        let message_data = env::prefix_contract_id(vec![]).await;
        let message = (123, message_data);
        let coin1 = (1_000_000, AssetId::default());
        let coin2 = (1_000_000, AssetId::default());
        let coin3 = (1_000_000, AssetId::default());

        // Set up the environment
        let (wallet, test_contract, contract_input, coin_inputs, message_inputs) =
            env::setup_environment(vec![coin1, coin2, coin3], vec![message]).await;

        // Relay the test message to the test contract
        let _receipts = env::relay_message_to_contract(
            &wallet,
            message_inputs[0].clone(),
            contract_input,
            &coin_inputs[..],
            &vec![],
            &vec![],
        )
        .await;

        // Verify test contract received the message
        let test_contract_counter = test_contract.get_test_counter().call().await.unwrap().value;
        assert_eq!(test_contract_counter, 1);

        // Verify test contract received the correct data
        let test_contract_id: ContractId = test_contract._get_contract_id().into();
        let test_contract_data1 = test_contract.get_test_data1().call().await.unwrap().value;
        assert_eq!(test_contract_data1, test_contract_id);

        // Verify the message value was received by the test contract
        let provider = wallet.get_provider().unwrap();
        let test_contract_balance = provider
            .get_contract_asset_balance(test_contract._get_contract_id(), AssetId::default())
            .await
            .unwrap();
        assert_eq!(test_contract_balance, 123);
    }

    #[tokio::test]
    async fn relay_message_with_optional_inputs_and_outputs() {
        let message_data = env::prefix_contract_id(vec![]).await;
        let message = (420, message_data);
        let coin = (DEFAULT_COIN_AMOUNT, AssetId::default());

        // Set up the environment
        let (wallet, test_contract, contract_input, coin_inputs, message_inputs) =
            env::setup_environment(vec![coin], vec![message]).await;

        // Deploy another contract for testing multiple contract inputs
        let other_contract_id = Contract::deploy_with_parameters(
            env::TEST_RECEIVER_CONTRACT_BINARY,
            &wallet,
            TxParameters::default(),
            StorageConfiguration::default(),
            Salt::from_str(RANDOM_SALT).unwrap(),
        )
        .await
        .unwrap();

        // Create optional inputs/outputs
        let input_contract = Input::Contract {
            utxo_id: UtxoId::new(Bytes32::zeroed(), 0u8),
            balance_root: Bytes32::zeroed(),
            state_root: Bytes32::zeroed(),
            tx_pointer: TxPointer::default(),
            contract_id: other_contract_id.into(),
        };
        let output_variable1 = Output::Variable {
            to: Address::default(),
            amount: Word::default(),
            asset_id: AssetId::default(),
        };
        let output_variable2 = Output::Variable {
            to: Address::default(),
            amount: Word::default(),
            asset_id: AssetId::from_str(RANDOM_SALT).unwrap(),
        };
        let output_contract = Output::Contract {
            input_index: 3u8,
            balance_root: Bytes32::zeroed(),
            state_root: Bytes32::zeroed(),
        };
        let output_message = Output::Message {
            recipient: Address::default(),
            amount: Word::default(),
        };

        // Relay the test message to the test contract
        let _receipts = env::relay_message_to_contract(
            &wallet,
            message_inputs[0].clone(),
            contract_input,
            &coin_inputs[..],
            &vec![input_contract],
            &vec![
                output_variable1,
                output_variable2,
                output_contract,
                output_message,
            ],
        )
        .await;

        // Verify test contract received the message
        let test_contract_counter = test_contract.get_test_counter().call().await.unwrap().value;
        assert_eq!(test_contract_counter, 1);

        // Verify test contract received the correct data
        let test_contract_id: ContractId = test_contract._get_contract_id().into();
        let test_contract_data1 = test_contract.get_test_data1().call().await.unwrap().value;
        assert_eq!(test_contract_data1, test_contract_id);

        // Verify the message value was received by the test contract
        let provider = wallet.get_provider().unwrap();
        let test_contract_balance = provider
            .get_contract_asset_balance(test_contract._get_contract_id(), AssetId::default())
            .await
            .unwrap();
        assert_eq!(test_contract_balance, 420);
    }
}

// Test the cases where the transaction should panic due to the
// predicate script failing to validate the transaction requirements
mod panic {
    use std::str::FromStr;

    use crate::utils::environment as env;
    use crate::utils::ext_sdk_provider;
    use fuels::prelude::Salt;
    use fuels::prelude::TxParameters;
    use fuels::test_helpers::DEFAULT_COIN_AMOUNT;
    use fuels::tx::{Address, AssetId, Input, Output, Transaction, TxPointer, UtxoId, Word};

    pub const RANDOM_SALT: &str =
        "0xf55c10bc8307552781a896ebd5e6d2b9278b4177b8bca400d3e7710eee293786";

    #[tokio::test]
    #[should_panic(expected = "The transaction contains a predicate which failed to validate")]
    async fn relay_message_with_too_many_inputs() {
        let message_data = env::prefix_contract_id(vec![]).await;
        let message = (100, message_data);
        let coin1 = (1_000_000, AssetId::default());
        let coin2 = (1_000_000, AssetId::default());
        let coin3 = (1_000_000, AssetId::default());
        let coin4 = (1_000_000, AssetId::default());
        let coin5 = (1_000_000, AssetId::default());
        let coin6 = (1_000_000, AssetId::default());
        let coin7 = (1_000_000, AssetId::default());

        // Set up the environment
        let (wallet, _, contract_input, coin_inputs, message_inputs) = env::setup_environment(
            vec![coin1, coin2, coin3, coin4, coin5, coin6, coin7],
            vec![message],
        )
        .await;

        // Build the message relaying transaction
        let mut tx = ext_sdk_provider::build_contract_message_tx(
            message_inputs[0].clone(),
            contract_input,
            &coin_inputs[..],
            &vec![],
            &vec![],
            TxParameters::default(),
        )
        .await;

        // Sign transaction and call
        // Note: tx inputs[contract, message, coin1, coin2, coin3, coin4, coin5, coin6, coin7], tx outputs[contract, change]
        let _receipts = env::sign_and_call_tx(&wallet, &mut tx).await;
    }

    #[tokio::test]
    #[should_panic(expected = "The transaction contains a predicate which failed to validate")]
    async fn relay_message_with_missing_input_contract() {
        let message_data = env::prefix_contract_id(vec![]).await;
        let message = (100, message_data);
        let coin1 = (DEFAULT_COIN_AMOUNT, AssetId::default());
        let coin2 = (DEFAULT_COIN_AMOUNT, AssetId::default());

        // Set up the environment
        let (wallet, _, contract_input, coin_inputs, message_inputs) =
            env::setup_environment(vec![coin1, coin2], vec![message]).await;

        // Build the message relaying transaction
        let mut tx = ext_sdk_provider::build_contract_message_tx(
            message_inputs[0].clone(),
            contract_input,
            &coin_inputs[..],
            &vec![],
            &vec![],
            TxParameters::default(),
        )
        .await;

        // Modify the transaction
        // Note: tx inputs[contract, message, coin1, coin2], tx outputs[contract, change]
        let inputs_length = tx.inputs().len();
        match &mut tx {
            Transaction::Script {
                inputs, outputs, ..
            } => {
                // Swap the input contract with 'coin2' at the end
                inputs.swap(0, inputs_length - 1);

                // Correct the input index for the contract output
                match &mut outputs[0] {
                    Output::Contract { input_index, .. } => {
                        *input_index = (inputs_length - 1) as u8;
                    }
                    _ => {}
                };
            }
            _ => {}
        };

        // Sign transaction and call
        // Note: tx inputs[coin2, message, coin1, contract], tx outputs[contract, change]
        let _receipts = env::sign_and_call_tx(&wallet, &mut tx).await;
    }

    #[tokio::test]
    //#[should_panic]
    #[should_panic(expected = "The transaction contains a predicate which failed to validate")]
    async fn relay_message_with_missing_input_message() {
        let coin = (DEFAULT_COIN_AMOUNT, AssetId::default());

        // Set up the environment
        let (wallet, _, contract_input, coin_inputs, _) =
            env::setup_environment(vec![coin], vec![]).await;

        // Transfer coins to a coin with the predicate as an owner
        let (predicate_bytecode, predicate_root) =
            ext_sdk_provider::get_contract_message_predicate().await;
        let _receipt = wallet
            .transfer(
                &predicate_root.into(),
                100,
                AssetId::default(),
                TxParameters::default(),
            )
            .await
            .unwrap();
        let predicate_coin = &wallet
            .get_provider()
            .unwrap()
            .get_coins(&predicate_root.into(), AssetId::default())
            .await
            .unwrap()[0];
        let coin_as_message = Input::CoinPredicate {
            utxo_id: UtxoId::from(predicate_coin.utxo_id.clone()),
            owner: predicate_root,
            amount: 100,
            asset_id: AssetId::default(),
            tx_pointer: TxPointer::default(),
            maturity: 0,
            predicate: predicate_bytecode,
            predicate_data: vec![],
        };

        // Build the message relaying transaction
        let mut tx = ext_sdk_provider::build_contract_message_tx(
            coin_as_message,
            contract_input,
            &coin_inputs,
            &vec![],
            &vec![],
            TxParameters::default(),
        )
        .await;

        // Sign transaction and call
        // Note: tx inputs[contract, coin_message, coin], tx outputs[contract, change]
        let _receipts = env::sign_and_call_tx(&wallet, &mut tx).await;
    }

    #[tokio::test]
    #[should_panic(expected = "The transaction contains a predicate which failed to validate")]
    async fn relay_message_with_mismatched_contract_ids() {
        let message_data_bad = Salt::from_str(RANDOM_SALT).unwrap().to_vec();
        let message = (100, message_data_bad);
        let coin = (1_000_000, AssetId::default());

        // Set up the environment
        let (wallet, _, contract_input, coin_inputs, message_inputs) =
            env::setup_environment(vec![coin], vec![message]).await;

        // Build the message relaying transaction
        let mut tx = ext_sdk_provider::build_contract_message_tx(
            message_inputs[0].clone(),
            contract_input,
            &coin_inputs[..],
            &vec![],
            &vec![],
            TxParameters::default(),
        )
        .await;

        // Sign transaction and call
        // Note: tx inputs[contract, message, coin], tx outputs[contract, change]
        let _receipts = env::sign_and_call_tx(&wallet, &mut tx).await;
    }

    #[tokio::test]
    #[should_panic(expected = "The transaction contains a predicate which failed to validate")]
    async fn relay_message_with_multiple_message_inputs() {
        let message_data1 = env::prefix_contract_id(vec![]).await;
        let message_data2 = Salt::from_str(RANDOM_SALT).unwrap().to_vec();
        let message1 = (100, message_data1);
        let message2 = (100, message_data2);
        let coin = (1_000_000, AssetId::default());

        // Set up the environment
        let (wallet, _, contract_input, coin_inputs, message_inputs) =
            env::setup_environment(vec![coin], vec![message1, message2]).await;

        // Build the message relaying transaction
        let mut tx = ext_sdk_provider::build_contract_message_tx(
            message_inputs[0].clone(),
            contract_input,
            &coin_inputs[..],
            &vec![message_inputs[1].clone()],
            &vec![],
            TxParameters::default(),
        )
        .await;

        // Sign transaction and call
        // Note: tx inputs[contract, message1, coin, message2], tx outputs[contract, change]
        let _receipts = env::sign_and_call_tx(&wallet, &mut tx).await;
    }

    #[tokio::test]
    #[should_panic(expected = "The transaction contains a predicate which failed to validate")]
    async fn relay_message_with_invalid_coin_assets() {
        let message_data = env::prefix_contract_id(vec![]).await;
        let message = (100, message_data);
        let coin1 = (1_000_000, AssetId::default());
        let coin2 = (1_000_000, AssetId::from_str(RANDOM_SALT).unwrap());

        // Set up the environment
        let (wallet, _, contract_input, coin_inputs, message_inputs) =
            env::setup_environment(vec![coin1, coin2], vec![message]).await;

        // Create a coin output to catch the value of the odd coin input
        let output_coin = Output::Coin {
            to: wallet.address().into(),
            amount: Word::from(coin2.0),
            asset_id: coin2.1,
        };

        // Build the message relaying transaction
        let mut tx = ext_sdk_provider::build_contract_message_tx(
            message_inputs[0].clone(),
            contract_input,
            &coin_inputs[..],
            &vec![],
            &vec![output_coin],
            TxParameters::default(),
        )
        .await;

        // Sign transaction and call
        // Note: tx inputs[contract, message, coin1, coin2], tx outputs[contract, change, coin2]
        let _receipts = env::sign_and_call_tx(&wallet, &mut tx).await;
    }

    #[tokio::test]
    #[should_panic(expected = "The transaction contains a predicate which failed to validate")]
    async fn relay_message_with_invalid_change_output() {
        let message_data = env::prefix_contract_id(vec![]).await;
        let message = (100, message_data);
        let coin1 = (1_000_000, AssetId::default());
        let coin2 = (1_000_000, AssetId::from_str(RANDOM_SALT).unwrap());

        // Set up the environment
        let (wallet, _, contract_input, coin_inputs, message_inputs) =
            env::setup_environment(vec![coin1, coin2], vec![message]).await;

        // Create a coin output to catch the value of the odd coin input
        let output_change = Output::Change {
            to: wallet.address().into(),
            amount: Word::default(),
            asset_id: coin2.1,
        };

        // Build the message relaying transaction
        let mut tx = ext_sdk_provider::build_contract_message_tx(
            message_inputs[0].clone(),
            contract_input,
            &coin_inputs[..],
            &vec![],
            &vec![output_change],
            TxParameters::default(),
        )
        .await;

        // Sign transaction and call
        // Note: tx inputs[contract, message, coin1, coin2], tx outputs[contract, change, change2]
        let _receipts = env::sign_and_call_tx(&wallet, &mut tx).await;
    }

    #[tokio::test]
    #[should_panic(expected = "The transaction contains a predicate which failed to validate")]
    async fn relay_message_with_missing_output_contract() {
        let message_data = env::prefix_contract_id(vec![]).await;
        let message = (100, message_data);
        let coin = (DEFAULT_COIN_AMOUNT, AssetId::default());

        // Set up the environment
        let (wallet, _, contract_input, coin_inputs, message_inputs) =
            env::setup_environment(vec![coin], vec![message]).await;

        // Create a variable output to sit in place of the contract input
        let output_variable = Output::Variable {
            to: Address::default(),
            amount: Word::default(),
            asset_id: AssetId::default(),
        };

        // Build the message relaying transaction
        let mut tx = ext_sdk_provider::build_contract_message_tx(
            message_inputs[0].clone(),
            contract_input,
            &coin_inputs[..],
            &vec![],
            &vec![output_variable],
            TxParameters::default(),
        )
        .await;

        // Modify the transaction
        let inputs_length = tx.inputs().len();
        match &mut tx {
            Transaction::Script { outputs, .. } => {
                // Swap the output contract at the start with the coutput variable at the end
                outputs.swap(0, inputs_length - 1);
            }
            _ => {}
        };

        // Sign transaction and call
        // Note: tx inputs[contract, message, coin], tx outputs[variable, change, contract]
        let _receipts = env::sign_and_call_tx(&wallet, &mut tx).await;
    }

    #[tokio::test]
    #[should_panic(expected = "The transaction contains a predicate which failed to validate")]
    async fn relay_message_with_missing_output_change() {
        let message_data = env::prefix_contract_id(vec![]).await;
        let message = (100, message_data);
        let coin = (DEFAULT_COIN_AMOUNT, AssetId::default());

        // Set up the environment
        let (wallet, _, contract_input, coin_inputs, message_inputs) =
            env::setup_environment(vec![coin], vec![message]).await;

        // Create a variable output to sit in place of the contract input
        let output_variable = Output::Variable {
            to: Address::default(),
            amount: Word::default(),
            asset_id: AssetId::default(),
        };

        // Build the message relaying transaction
        let mut tx = ext_sdk_provider::build_contract_message_tx(
            message_inputs[0].clone(),
            contract_input,
            &coin_inputs[..],
            &vec![],
            &vec![output_variable],
            TxParameters::default(),
        )
        .await;

        // Modify the transaction
        let inputs_length = tx.inputs().len();
        match &mut tx {
            Transaction::Script { outputs, .. } => {
                // Swap the output change with the output variable at the end
                outputs.swap(1, inputs_length - 1);
            }
            _ => {}
        };

        // Sign transaction and call
        // Note: tx inputs[contract, message, coin], tx outputs[contract, variable, change]
        let _receipts = env::sign_and_call_tx(&wallet, &mut tx).await;
    }

    #[tokio::test]
    #[should_panic(expected = "The transaction contains a predicate which failed to validate")]
    async fn relay_message_with_too_many_outputs() {
        let message_data = env::prefix_contract_id(vec![]).await;
        let message = (100, message_data);
        let coin = (DEFAULT_COIN_AMOUNT, AssetId::default());

        // Set up the environment
        let (wallet, _, contract_input, coin_inputs, message_inputs) =
            env::setup_environment(vec![coin], vec![message]).await;

        // Create 7 output messages to include in tx
        let output_messages: Vec<Output> = (0..7)
            .map(|_i| Output::Message {
                recipient: Address::default(),
                amount: Word::default(),
            })
            .collect();

        // Build the message relaying transaction
        let mut tx = ext_sdk_provider::build_contract_message_tx(
            message_inputs[0].clone(),
            contract_input,
            &coin_inputs[..],
            &vec![],
            &output_messages,
            TxParameters::default(),
        )
        .await;

        // Sign transaction and call
        // Note: tx inputs[contract, message, coin], tx outputs[contract, change, message1, message2, message3, message4, message5, message6, message7]
        let _receipts = env::sign_and_call_tx(&wallet, &mut tx).await;
    }

    #[tokio::test]
    #[should_panic(expected = "The transaction contains a predicate which failed to validate")]
    async fn relay_message_with_too_little_gas() {
        let message_data = env::prefix_contract_id(vec![]).await;
        let message = (100, message_data);
        let coin = (1_000_000, AssetId::default());

        // Set up the environment
        let (wallet, _, contract_input, coin_inputs, message_inputs) =
            env::setup_environment(vec![coin], vec![message]).await;

        // Build the message relaying transaction
        let mut tx = ext_sdk_provider::build_contract_message_tx(
            message_inputs[0].clone(),
            contract_input,
            &coin_inputs[..],
            &vec![],
            &vec![],
            TxParameters::default(),
        )
        .await;

        // Sign transaction and call
        // Note: tx inputs[contract, message, coin], tx outputs[contract, change]
        let _receipts = env::sign_and_call_tx(&wallet, &mut tx).await;
    }

    #[tokio::test]
    #[should_panic(expected = "The transaction contains a predicate which failed to validate")]
    async fn relay_message_with_invalid_script() {
        let message_data = env::prefix_contract_id(vec![]).await;
        let message = (100, message_data);
        let coin = (DEFAULT_COIN_AMOUNT, AssetId::default());

        // Set up the environment
        let (wallet, _, contract_input, coin_inputs, message_inputs) =
            env::setup_environment(vec![coin], vec![message]).await;

        // Build the message relaying transaction
        let mut tx = ext_sdk_provider::build_contract_message_tx(
            message_inputs[0].clone(),
            contract_input,
            &coin_inputs[..],
            &vec![],
            &vec![],
            TxParameters::default(),
        )
        .await;

        // Modify the script bytecode
        match &mut tx {
            Transaction::Script { script, .. } => {
                *script = vec![0u8, 1u8, 2u8, 3u8];
            }
            _ => {}
        }

        // Sign transaction and call
        // Note: tx inputs[contract, message, coin], tx outputs[contract, change]
        let _receipts = env::sign_and_call_tx(&wallet, &mut tx).await;
    }
}
