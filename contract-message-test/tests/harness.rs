mod utils {
    pub mod environment;
    pub mod ext_fuel_core;
    pub mod ext_sdk_provider;
}
use utils::environment as env;
use utils::ext_fuel_core;
use utils::ext_sdk_provider;

use fuels::test_helpers::DEFAULT_COIN_AMOUNT;
use fuels::tx::{ContractId, AssetId};

#[tokio::test]
async fn report_data() {
    // Log useful details about the message-to-contract script and predicate
    let (_, script_hash) = ext_sdk_provider::get_contract_message_script().await;
    let (_, predicate_root) = ext_sdk_provider::get_contract_message_predicate().await;
    println!("Script Hash : 0x{:?}", script_hash);
    println!("Predicate Root : 0x{:?}", predicate_root);
}

#[tokio::test]
async fn relay_message_with_predicate_and_script_constraint() {
    let message_data = env::prefix_contract_id(vec![]).await;
    let message = (100, message_data);
    let coin = (DEFAULT_COIN_AMOUNT, AssetId::default());

    // Set up the environment
    let (wallet, test_contract, coin_inputs, message_inputs) =
        env::setup_environment(vec![coin.clone()], vec![message.clone()]).await;

    // Relay the test message to the test contract
    let _receipts = ext_sdk_provider::relay_test_contract_message(
        &wallet,
        message.clone(),
        test_contract._get_contract_id().into(),
    )
    .await;

    //////////////////////////////////////////////////////////////////////////
    let provider = wallet.get_provider().unwrap();
    let test_contract_id: ContractId = test_contract._get_contract_id().into();
    println!("test_contract_id: {:#?}", test_contract_id);
    let test_contract_data1 = test_contract.get_test_data1().call().await.unwrap().value;
    println!("test_contract_data1: {:#?}", test_contract_data1);
    let test_contract_balance = provider.get_contract_asset_balance(test_contract._get_contract_id(), AssetId::default()).await.unwrap();
    println!("test_contract_balance: {:#?}", test_contract_balance);

    // Verify test contract counter was incremented
    let test_contract_counter = test_contract.get_test_counter().call().await.unwrap().value;
    assert_eq!(test_contract_counter, 1);
}
