mod utils {
    pub mod builder;
}

use utils::builder;
use fuels::prelude::*;

#[tokio::main]
async fn main() {

    // Connect to provider
    let provider = Provider::connect("node-beta-1.fuel.network").await.unwrap();

    // Unlock wallet
    // TO DO 

    // Get bytecode of predicate and its root
    let (_predicate_bytecode, predicate_root) = builder::get_contract_message_predicate().await;

    // Get unspend messages belonging to the bytecode root
    let unspent_messages = provider.get_messages(&Bech32Address::from(predicate_root)).await.unwrap();

    for message in unspent_messages {
        //builder::relay_message_to_contract(wallet: &WalletUnlocked, message: Input);
    }

}
