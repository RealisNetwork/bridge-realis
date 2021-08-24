#[cfg(test)]
mod accounts;

#[cfg(test)]
mod tests {
    use self::super::accounts;
    use futures::StreamExt;
    use log::info;
    use ratsio::{StanClient, StanOptions};
    use realis_primitives::TokenId;
    use runtime::{
        pallet_balances::Call as PalletBalancesCall,
        pallet_nft::Call as PalletNftCall, realis_bridge::Call as RealisBridgeCall,
        AccountId, Call,
    };
    use secp256k1::SecretKey;
    use serde_json::{self, Value};
    use sp_core::{H160, H256 as Hash};
    use sp_runtime::{generic, traits::BlakeTwo256};
    use std::{assert_eq, fs, sync::Arc};
    use substrate_api_client::{
        compose_extrinsic_offline,
        sp_runtime::app_crypto::{sr25519, Pair},
        Api, BlockNumber, XtStatus,
    };
    use tokio::time::{sleep, Duration};
    use utils;
    use web3::types::{Address, U256};

    type Header = generic::Header<BlockNumber, BlakeTwo256>;

    // fn api(signer: sr25519::Pair) -> Api<sr25519::Pair> {
    //     // Url to realis blockchain
    //     let url = "rpc.realis.network";
    //     // Create substrate api with signer
    //     Api::<sr25519::Pair>::new(format!("wss://{}", String::from(url)))
    //         .map(|api| api.set_signer(signer))
    //         .unwrap()
    // }

    pub fn logger_setup() {
        use env_logger::Builder;
        use log::{info, LevelFilter};
        use std::io::Write;

        let _ = Builder::new()
            .format(|buf, record| {
                writeln!(buf, "[{}] - {}", record.level(), record.args())
            })
            .filter(None, LevelFilter::Trace)
            .try_init();
    }

    async fn get_stan_client() -> Arc<StanClient> {
        logger_setup();
        // Create stan options
        info!("Start connect to NATS_Streaming!");
        let client_id = "realis-bridge-test".to_string();
        let opts = StanOptions::with_options(
            "localhost:4222",
            "test-cluster",
            &client_id[..],
        );
        // Create STAN client
        let stan_client = StanClient::from_options(opts).await.unwrap();
        stan_client
    }

    async fn get_response(stan_client: &Arc<StanClient>, subject: String) -> Value {
        logger_setup();
        let mut subscription =
            stan_client.subscribe(subject, None, None).await.unwrap();

        let stan_message = subscription.1.next().await.unwrap();

        let message_string =
            String::from_utf8_lossy(stan_message.payload.as_ref()).into_owned();

        let json: Value = serde_json::from_str(&message_string).unwrap();

        json
    }

    // fn send_token_from_realis_to_bsc(
    //     signer: sr25519::Pair,
    //     from: AccountId,
    //     to: Address,
    //     amount: u128,
    // ) {
    //     let api = api(signer);
    //     // Create some parameters for transaction
    //     let head: Hash = api.get_finalized_head().unwrap().unwrap();
    //     let h: Header = api.get_header(Some(head)).unwrap().unwrap();
    //     let period = 5;
    //     // Create transaction
    //     #[allow(clippy::redundant_clone)]
    //     let xt = compose_extrinsic_offline!(
    //         api.clone().signer.unwrap(),
    //         Call::RealisBridge(RealisBridgeCall::transfer_token_to_bsc(
    //             from.clone(),
    //             H160::from(to.0),
    //             amount
    //         )),
    //         api.get_nonce().unwrap(),
    //         Era::mortal(period, h.number),
    //         api.genesis_hash,
    //         head,
    //         api.runtime_version.spec_version,
    //         api.runtime_version.transaction_version
    //     );
    //     // Send extrinsic transaction
    //     let _ = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);
    // }
    //
    // async fn transfer_token_in_bsc(
    //     signer: SecretKey,
    //     _from: Address,
    //     to: Address,
    //     amount: u128,
    // ) {
    //     // Connect to bsc smart contract
    //     let contract = utils::contract::token_new().await;
    //     // Send transaction
    //     let _ = contract
    //         .signed_call_with_confirmations(
    //             "transfer",
    //             (to, amount * 10_000_000_000),
    //             web3::contract::Options::default(),
    //             1,
    //             &signer,
    //         )
    //         .await;
    //     // Wait for transaction end
    //     sleep(Duration::from_millis(10_000)).await;
    // }
    //
    // #[test]
    // fn simple_token_transfer() {
    //     // Get Alice-realis account
    //     let (account_id, private) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (address, _) = accounts::bsc::alice();
    //     // Alice-realis transfer 1 token to Alice-bsc
    //     send_token_from_realis_to_bsc(private, account_id, address, 1);
    // }
    //
    // #[test]
    // fn transfer_some_tokens() {
    //     // Get Alice-realis account
    //     let (account_id, private) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (address, _) = accounts::bsc::alice();
    //     // Alice-realis transfer 1000 tokens to Alice-bsc
    //     send_token_from_realis_to_bsc(private, account_id, address, 1000);
    // }
    //
    // #[tokio::test]
    // async fn transfer_some_tokens_than_use_them_1() {
    //     // Get Alice-realis account
    //     let (account_id, private_realis) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (alice_address, private_bsc) = accounts::bsc::alice();
    //     // Get Bob-bsc account
    //     let (bob_address, _) = accounts::bsc::bob();
    //     // Alice-realis transfer 1000 tokens to Alice-bsc
    //     send_token_from_realis_to_bsc(
    //         private_realis,
    //         account_id,
    //         alice_address,
    //         1000,
    //     );
    //     // Alice-bsc transfer 1000 tokens to Bob-bsc
    //     transfer_token_in_bsc(private_bsc, alice_address, bob_address, 1000)
    //         .await;
    // }
    //
    // #[tokio::test]
    // async fn transfer_some_tokens_than_use_them_2() {
    //     // Get Alice-realis account
    //     let (account_id, private_realis) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (alice_address, private_bsc) = accounts::bsc::alice();
    //     // Get Bob-bsc account
    //     let (bob_address, _) = accounts::bsc::bob();
    //     // Get Cindy-bsc account
    //     let (cindy_address, _) = accounts::bsc::cindy();
    //     // Alice-realis transfer 1000 tokens to Alice-bsc
    //     send_token_from_realis_to_bsc(
    //         private_realis,
    //         account_id,
    //         alice_address,
    //         1000,
    //     );
    //     // Alice-bsc transfer 300 tokens to Bob-bsc
    //     transfer_token_in_bsc(private_bsc, alice_address, bob_address,
    // 300).await;     // Alice-bsc transfer 300 tokens to Cindy-bsc
    //     transfer_token_in_bsc(private_bsc, alice_address, cindy_address, 300)
    //         .await;
    // }
    //
    // #[tokio::test]
    // async fn transfer_some_tokens_than_use_them_3() {
    //     // Get Alice-realis account
    //     let (account_id, private_realis) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (alice_address, private_alice_bsc) = accounts::bsc::alice();
    //     // Get Bob-bsc account
    //     let (bob_address, private_bob_bsc) = accounts::bsc::bob();
    //     // Get Cindy-bsc account
    //     let (cindy_address, _) = accounts::bsc::cindy();
    //     // Alice-realis transfer 1000 tokens to Alice-bsc
    //     send_token_from_realis_to_bsc(
    //         private_realis,
    //         account_id,
    //         alice_address,
    //         1000,
    //     );
    //     // Wait for transaction end
    //     sleep(Duration::from_millis(10_000)).await;
    //     // Alice-bsc transfer 700 tokens to Bob-bsc
    //     transfer_token_in_bsc(private_alice_bsc, alice_address, bob_address,
    // 700)         .await;
    //     // Alice-bsc transfer 400 tokens to Cindy-bsc
    //     transfer_token_in_bsc(private_bob_bsc, bob_address, cindy_address, 400)
    //         .await;
    // }
    //
    // async fn send_nft_from_realis_to_bsc(
    //     signer: sr25519::Pair,
    //     from: AccountId,
    //     to: Address,
    //     token_id: TokenId,
    // ) {
    //     let api = api(signer);
    //     // Create some parameters for transaction
    //     let head: Hash = api.get_finalized_head().unwrap().unwrap();
    //     let h: Header = api.get_header(Some(head)).unwrap().unwrap();
    //     let period = 5;
    //     // Create transaction
    //     #[allow(clippy::redundant_clone)]
    //     let xt = compose_extrinsic_offline!(
    //         api.clone().signer.unwrap(),
    //         Call::RealisBridge(RealisBridgeCall::transfer_nft_to_bsc(
    //             from.clone(),
    //             H160::from(to.0),
    //             token_id
    //         )),
    //         api.get_nonce().unwrap(),
    //         Era::mortal(period, h.number),
    //         api.genesis_hash,
    //         head,
    //         api.runtime_version.spec_version,
    //         api.runtime_version.transaction_version
    //     );
    //     // Send extrinsic transaction
    //     let _ = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);
    //     // Wait for transaction end
    //     sleep(Duration::from_millis(20_000)).await;
    // }
    //
    // // async fn transfer_nft_in_bsc(
    // //     signer: SecretKey,
    // //     from: Address,
    // //     to: Address,
    // //     token_id: TokenId,
    // // ) {
    // //     // Connect to bsc smart contract
    // //     let contract = utils::contract::nft_new().await;
    // //     // Send transaction
    // //     let _ = contract
    // //         .signed_call_with_confirmations(
    // //             "TransferFrom",
    // //             (from, to, token_id),
    // //             web3::contract::Options::default(),
    // //             1,
    // //             &signer,
    // //         )
    // //         .await;
    // //     // Wait for transaction end
    // //     sleep(Duration::from_millis(15_000)).await;
    // // }
    //
    // #[tokio::test]
    // async fn simple_nft_transfer() {
    //     // Get Alice-realis account
    //     let (account_id, private) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (address, _) = accounts::bsc::alice();
    //     // Alice-realis transfer nft-11 to Alice-bsc
    //     send_nft_from_realis_to_bsc(
    //         private,
    //         account_id,
    //         address,
    //         TokenId::from(11),
    //     )
    //     .await;
    // }
    //
    // #[tokio::test]
    // async fn transfer_some_nft() {
    //     // Get Alice-realis account
    //     let (account_id, private) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (address, _) = accounts::bsc::alice();
    //     // Alice-realis transfer nft-21 to Alice-bsc
    //     send_nft_from_realis_to_bsc(
    //         private.clone(),
    //         account_id.clone(),
    //         address,
    //         TokenId::from(21).into(),
    //     )
    //     .await;
    //     // Alice-realis transfer nft-22 to Alice-bsc
    //     send_nft_from_realis_to_bsc(
    //         private,
    //         account_id,
    //         address,
    //         TokenId::from(22).into(),
    //     )
    //     .await;
    // }

    // #[tokio::test]
    // async fn transfer_some_nft_than_use_them_1() {
    //     // Get Alice-realis account
    //     let (account_id, private) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (alice_address, private_alice_bsc) = accounts::bsc::alice();
    //     // Get Bob-bsc account
    //     let (bob_address, private_bob_bsc) = accounts::bsc::bob();
    //     // Alice-realis transfer nft-31 to Alice-bsc
    //     send_nft_from_realis_to_bsc(
    //         private.clone(),
    //         account_id.clone(),
    //         alice_address,
    //         TokenId::from(31).into(),
    //     )
    //     .await;
    //     // Alice-bsc transfer nft-31 to Bob-bsc
    //     transfer_nft_in_bsc(
    //         private_alice_bsc,
    //         alice_address,
    //         bob_address,
    //         TokenId::from(31).into(),
    //     )
    //     .await;
    //
    //     // Bob-bsc transfer nft-31 to Alice-bsc
    //     transfer_nft_in_bsc(
    //         private_bob_bsc,
    //         bob_address,
    //         alice_address,
    //         TokenId::from(31).into(),
    //     )
    //     .await;
    // }
    //
    // #[tokio::test]
    // async fn transfer_some_nft_than_use_them_2() {
    //     // Get Alice-realis account
    //     let (account_id, private) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (alice_address, private_alice_bsc) = accounts::bsc::alice();
    //     // Get Bob-bsc account
    //     let (bob_address, private_bob_bsc) = accounts::bsc::bob();
    //     // Get Cindy-bsc account
    //     let (cindy_address, private_cindy_bsc) = accounts::bsc::cindy();
    //     // Alice-realis transfer nft-41 to Alice-bsc
    //     send_nft_from_realis_to_bsc(
    //         private.clone(),
    //         account_id.clone(),
    //         alice_address,
    //         TokenId::from(41).into(),
    //     )
    //     .await;
    //     // Alice-realis transfer nft-42 to Alice-bsc
    //     send_nft_from_realis_to_bsc(
    //         private.clone(),
    //         account_id.clone(),
    //         alice_address,
    //         TokenId::from(42).into(),
    //     )
    //     .await;
    //     // Alice-bsc transfer nft-41 to Bob-bsc
    //     transfer_nft_in_bsc(
    //         private_alice_bsc,
    //         alice_address,
    //         bob_address,
    //         TokenId::from(41).into(),
    //     )
    //     .await;
    //     // Alice-bsc transfer nft-42 to Cindy-bsc
    //     transfer_nft_in_bsc(
    //         private_alice_bsc,
    //         alice_address,
    //         cindy_address,
    //         TokenId::from(42).into(),
    //     )
    //     .await;
    //
    //     // Bob-bsc transfer nft-41 to Alice-bsc
    //     transfer_nft_in_bsc(
    //         private_bob_bsc,
    //         bob_address,
    //         alice_address,
    //         TokenId::from(41).into(),
    //     )
    //     .await;
    //     // Cindy-bsc transfer nft-42 to Alice-bsc
    //     transfer_nft_in_bsc(
    //         private_cindy_bsc,
    //         cindy_address,
    //         alice_address,
    //         TokenId::from(42).into(),
    //     )
    //     .await;
    // }
    //
    // #[tokio::test]
    // async fn transfer_some_nft_than_use_them_3() {
    //     // Get Alice-realis account
    //     let (account_id, private) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (alice_address, private_alice_bsc) = accounts::bsc::alice();
    //     // Get Bob-bsc account
    //     let (bob_address, private_bob_bsc) = accounts::bsc::bob();
    //     // Get Cindy-bsc account
    //     let (cindy_address, private_cindy_bsc) = accounts::bsc::cindy();
    //     // Alice-realis transfer nft-51 to Alice-bsc
    //     send_nft_from_realis_to_bsc(
    //         private.clone(),
    //         account_id.clone(),
    //         alice_address,
    //         TokenId::from(51).into(),
    //     )
    //     .await;
    //     // Alice-realis transfer nft-52 to Alice-bsc
    //     send_nft_from_realis_to_bsc(
    //         private.clone(),
    //         account_id.clone(),
    //         alice_address,
    //         TokenId::from(52).into(),
    //     )
    //     .await;
    //     // Alice-bsc transfer nft-51 to Bob-bsc
    //     transfer_nft_in_bsc(
    //         private_alice_bsc,
    //         alice_address,
    //         bob_address,
    //         TokenId::from(51).into(),
    //     )
    //     .await;
    //     // Alice-bsc transfer nft-52 to Bob-bsc
    //     transfer_nft_in_bsc(
    //         private_alice_bsc,
    //         alice_address,
    //         bob_address,
    //         TokenId::from(52).into(),
    //     )
    //     .await;
    //     // Bob-bsc transfer nft-51 to Cindy-bsc
    //     transfer_nft_in_bsc(
    //         private_bob_bsc,
    //         bob_address,
    //         cindy_address,
    //         TokenId::from(51).into(),
    //     )
    //     .await;
    //
    //     // Bob-bsc transfer nft-52 to Alice-bsc
    //     transfer_nft_in_bsc(
    //         private_bob_bsc,
    //         bob_address,
    //         alice_address,
    //         TokenId::from(52).into(),
    //     )
    //     .await;
    //     // Cindy-bsc transfer nft-51 to Alice-bsc
    //     transfer_nft_in_bsc(
    //         private_cindy_bsc,
    //         cindy_address,
    //         alice_address,
    //         TokenId::from(51).into(),
    //     )
    //     .await;
    // }
    //
    // async fn send_tokens_from_bsc_to_realis(
    //     signer: SecretKey,
    //     _from: Address,
    //     to: AccountId,
    //     amount: u128,
    // ) {
    //     // Connect to bsc smart contract
    //     let contract = utils::contract::token_new().await;
    //     // Send transaction
    //     let _ = contract
    //         .signed_call_with_confirmations(
    //             "transferToRealis",
    //             (to.to_string(), U256::from(amount)),
    //             web3::contract::Options::default(),
    //             1,
    //             &signer,
    //         )
    //         .await;
    //     // Wait for transaction end
    //     sleep(Duration::from_millis(10_000)).await;
    // }
    //
    // fn transfer_tokens_in_realis(
    //     signer: sr25519::Pair,
    //     _from: AccountId,
    //     to: AccountId,
    //     amount: u128,
    // ) {
    // let api = api(signer);
    // // Create some parameters for transaction
    // let head: Hash = api.get_finalized_head().unwrap().unwrap();
    // let h: Header = api.get_header(Some(head)).unwrap().unwrap();
    // let period = 5;
    // // Create transaction
    // #[allow(clippy::redundant_clone)]
    // let xt = compose_extrinsic_offline!(
    //     api.clone().signer.unwrap(),
    //     Call::Balances(PalletBalancesCall::transfer(
    //         to,
    //         amount
    //     )),
    //     api.get_nonce().unwrap(),
    //     Era::mortal(period, h.number),
    //     api.genesis_hash,
    //     head,
    //     api.runtime_version.spec_version,
    //     api.runtime_version.transaction_version
    // );
    // // Send extrinsic transaction
    // let _ = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);
    // }

    // #[tokio::test]
    // async fn simple_token_transfer_to_realis() {
    //     // Get Alice-realis account
    //     let (account_id, _) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (address, private) = accounts::bsc::alice();
    //     // Alice-bsc transfer 1 token to Alice-realis
    //     let _ =
    //         send_tokens_from_bsc_to_realis(private, address, account_id,
    // 1).await; }
    //
    // #[tokio::test]
    // async fn transfer_some_tokens_to_realis() {
    //     // Get Alice-realis account
    //     let (account_id, _) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (address, private) = accounts::bsc::alice();
    //     // Alice-bsc transfer 1000 tokens to Alice-realis
    //     let _ = send_tokens_from_bsc_to_realis(
    //         private,
    //         address,
    //         account_id.clone(),
    //         1000,
    //     )
    //     .await;
    // }
    //
    // #[tokio::test]
    // async fn transfer_some_tokens_to_realis_than_use_them_1() {
    //     // Get Alice-realis account
    //     let (alice_account_id, private_alice_realis) =
    // accounts::realis::alice();     // Get Bob-realis account
    //     let (bob_account_id, private_bob_realis) = accounts::realis::bob();
    //     // Get Alice-bsc account
    //     let (address, private_bsc) = accounts::bsc::alice();
    //     // Alice-bsc transfer 1000 tokens to Alice-realis
    //     let _ = send_tokens_from_bsc_to_realis(
    //         private_bsc,
    //         address,
    //         alice_account_id.clone(),
    //         1000,
    //     )
    //     .await;
    //     // Alice-realis transfer 1000 tokens to Bob-realis
    //     transfer_tokens_in_realis(
    //         private_alice_realis,
    //         alice_account_id.clone(),
    //         bob_account_id.clone(),
    //         1000,
    //     );
    //
    //     // Bob-realis transfer 1000 tokens to Alice-realis
    //     transfer_tokens_in_realis(
    //         private_bob_realis,
    //         bob_account_id,
    //         alice_account_id,
    //         1000,
    //     );
    // }
    //
    // #[tokio::test]
    // async fn transfer_some_tokens_to_realis_than_use_them_2() {
    //     // Get Alice-realis account
    //     let (alice_account_id, private_alice_realis) =
    // accounts::realis::alice();     // Get Bob-realis account
    //     let (bob_account_id, private_bob_realis) = accounts::realis::bob();
    //     // Get Cindy-realis account
    //     let (cindy_account_id, private_cindy_realis) =
    // accounts::realis::cindy();     // Get Alice-bsc account
    //     let (address, private_bsc) = accounts::bsc::alice();
    //     // Alice-bsc transfer 1000 tokens to Alice-realis
    //     let _ = send_tokens_from_bsc_to_realis(
    //         private_bsc,
    //         address,
    //         alice_account_id.clone(),
    //         1000,
    //     )
    //     .await;
    //     // Alice-realis transfer 300 tokens to Bob-realis
    //     transfer_tokens_in_realis(
    //         private_alice_realis.clone(),
    //         alice_account_id.clone(),
    //         bob_account_id.clone(),
    //         300,
    //     );
    //     // Alice-realis transfer 300 tokens to Cindy-realis
    //     transfer_tokens_in_realis(
    //         private_alice_realis,
    //         alice_account_id.clone(),
    //         cindy_account_id.clone(),
    //         300,
    //     );
    //
    //     // Bob-realis transfer 300 tokens to Alice-realis
    //     transfer_tokens_in_realis(
    //         private_bob_realis,
    //         bob_account_id,
    //         alice_account_id.clone(),
    //         300,
    //     );
    //     // Cindy-realis transfer 300 tokens to Alice-realis
    //     transfer_tokens_in_realis(
    //         private_cindy_realis,
    //         cindy_account_id,
    //         alice_account_id,
    //         300,
    //     );
    // }
    //
    // #[tokio::test]
    // async fn transfer_some_tokens_to_realis_than_use_them_3() {
    //     // Get Alice-realis account
    //     let (alice_account_id, private_alice_realis) =
    // accounts::realis::alice();     // Get Bob-realis account
    //     let (bob_account_id, private_bob_realis) = accounts::realis::bob();
    //     // Get Cindy-realis account
    //     let (cindy_account_id, private_cindy_realis) =
    // accounts::realis::cindy();     // Get Alice-bsc account
    //     let (address, private_bsc) = accounts::bsc::alice();
    //     // Alice-bsc transfer 1000 tokens to Alice-realis
    //     let _ = send_tokens_from_bsc_to_realis(
    //         private_bsc,
    //         address,
    //         alice_account_id.clone(),
    //         1000,
    //     )
    //     .await;
    //     // Alice-realis transfer 700 tokens to Bob-realis
    //     transfer_tokens_in_realis(
    //         private_alice_realis.clone(),
    //         alice_account_id.clone(),
    //         bob_account_id.clone(),
    //         700,
    //     );
    //     // Bob-realis transfer 400 tokens to Cindy-realis
    //     transfer_tokens_in_realis(
    //         private_bob_realis.clone(),
    //         bob_account_id.clone(),
    //         cindy_account_id.clone(),
    //         400,
    //     );
    //
    //     // Bob-realis transfer 300 tokens to Alice-realis
    //     transfer_tokens_in_realis(
    //         private_bob_realis,
    //         bob_account_id,
    //         alice_account_id.clone(),
    //         300,
    //     );
    //     // Cindy-realis transfer 400 tokens to Alice-realis
    //     transfer_tokens_in_realis(
    //         private_cindy_realis,
    //         cindy_account_id,
    //         alice_account_id,
    //         400,
    //     );
    // }
    //
    // // async fn send_nft_from_bsc_to_realis(
    // //     signer: SecretKey,
    // //     _from: Address,
    // //     to: AccountId,
    // //     token_id: TokenId,
    // // ) {
    // //     // Connect to bsc smart contract
    // //     let contract = utils::contract::nft_new().await;
    // //     // Send transaction
    // //     let _ = contract
    // //         .signed_call_with_confirmations(
    // //             "transferToRealis",
    // //             (to.to_string(), token_id),
    // //             web3::contract::Options::default(),
    // //             1,
    // //             &signer,
    // //         )
    // //         .await;
    // //     // Wait for transaction end
    // //     sleep(Duration::from_millis(10_000)).await;
    // // }
    //
    // // fn transfer_nft_in_realis(
    // //     signer: sr25519::Pair,
    // //     _from: AccountId,
    // //     to: AccountId,
    // //     token_id: TokenId,
    // // ) {
    // //     let api = api(signer);
    // //     // Create some parameters for transaction
    // //     let head: Hash = api.get_finalized_head().unwrap().unwrap();
    // //     let h: Header = api.get_header(Some(head)).unwrap().unwrap();
    // //     let period = 5;
    // //     // Create transaction
    // //     #[allow(clippy::redundant_clone)]
    // //     let xt = compose_extrinsic_offline!(
    // //         api.clone().signer.unwrap(),
    // //         Call::Nft(PalletNftCall::transfer_basic(to.clone(), token_id)),
    // //         api.get_nonce().unwrap(),
    // //         Era::mortal(period, h.number),
    // //         api.genesis_hash,
    // //         head,
    // //         api.runtime_version.spec_version,
    // //         api.runtime_version.transaction_version
    // //     );
    // //     // Send extrinsic transaction
    // //     let _ = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);
    // // }
    //
    // #[tokio::test]
    // async fn simple_nft_transfer_to_bsc() {
    //     // Get Alice-realis account
    //     let (account_id, _) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (address, private) = accounts::bsc::alice();
    //     // Alice-bsc transfer nft-21 to Alice-realis
    //     let _ = send_nft_from_bsc_to_realis(
    //         private,
    //         address,
    //         account_id,
    //         TokenId::from(11),
    //     )
    //     .await;
    // }
    //
    // #[tokio::test]
    // async fn transfer_some_nft_to_realis() {
    //     // Get Alice-realis account
    //     let (account_id, _) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (address, private) = accounts::bsc::alice();
    //     // Alice-bsc transfer nft-21 to Alice-realis
    //     let _ = send_nft_from_bsc_to_realis(
    //         private,
    //         address,
    //         account_id.clone(),
    //         TokenId::from(21),
    //     )
    //     .await;
    //     // Alice-bsc transfer nft-22 to Alice-realis
    //     let _ = send_nft_from_bsc_to_realis(
    //         private,
    //         address,
    //         account_id,
    //         TokenId::from(22),
    //     )
    //     .await;
    // }
    //
    // #[tokio::test]
    // async fn transfer_some_nfr_to_realis_than_use_them_1() {
    //     // Get Alice-realis account
    //     let (alice_account_id, private_alice_realis) =
    // accounts::realis::alice();     // Get Bob-realis account
    //     let (bob_account_id, private_bob_realis) = accounts::realis::bob();
    //     // Get Alice-bsc account
    //     let (address, private_bsc) = accounts::bsc::alice();
    //     // Alice-bsc transfer nft-31 to Alice-realis
    //     let _ = send_nft_from_bsc_to_realis(
    //         private_bsc,
    //         address,
    //         alice_account_id.clone(),
    //         TokenId::from(31),
    //     )
    //     .await;
    //     // Alice-realis transfer nft-31 to Bob-realis
    //     transfer_nft_in_realis(
    //         private_alice_realis,
    //         alice_account_id.clone(),
    //         bob_account_id.clone(),
    //         TokenId::from(31),
    //     );
    //
    //     // Bob-realis transfer nft-31 to Alice-realis
    //     transfer_nft_in_realis(
    //         private_bob_realis,
    //         bob_account_id,
    //         alice_account_id.clone(),
    //         TokenId::from(31),
    //     );
    // }
    //
    // #[tokio::test]
    // async fn transfer_some_nfr_to_realis_than_use_them_2() {
    //     // Get Alice-realis account
    //     let (alice_account_id, private_alice_realis) =
    // accounts::realis::alice();     // Get Bob-realis account
    //     let (bob_account_id, private_bob_realis) = accounts::realis::bob();
    //     // Get Cindy-realis account
    //     let (cindy_account_id, private_cindy_realis) =
    // accounts::realis::cindy();     // Get Alice-bsc account
    //     let (address, private_bsc) = accounts::bsc::alice();
    //     // Alice-bsc transfer nft-41 to Alice-realis
    //     let _ = send_nft_from_bsc_to_realis(
    //         private_bsc,
    //         address,
    //         alice_account_id.clone(),
    //         TokenId::from(41),
    //     )
    //     .await;
    //     // Alice-bsc transfer nft-42 to Alice-realis
    //     let _ = send_nft_from_bsc_to_realis(
    //         private_bsc,
    //         address,
    //         alice_account_id.clone(),
    //         TokenId::from(42),
    //     )
    //     .await;
    //     // Alice-realis transfer nft-41 to Bob-realis
    //     transfer_nft_in_realis(
    //         private_alice_realis.clone(),
    //         alice_account_id.clone(),
    //         bob_account_id.clone(),
    //         TokenId::from(41),
    //     );
    //     // Alice-realis transfer nft-42 to Cindy-realis
    //     transfer_nft_in_realis(
    //         private_alice_realis,
    //         alice_account_id.clone(),
    //         cindy_account_id.clone(),
    //         TokenId::from(42),
    //     );
    //
    //     // Bob-realis transfer nft-41 to Alice-realis
    //     transfer_nft_in_realis(
    //         private_bob_realis,
    //         bob_account_id,
    //         alice_account_id.clone(),
    //         TokenId::from(41),
    //     );
    //     // Cindy-realis transfer nft-42 to Alice-realis
    //     transfer_nft_in_realis(
    //         private_cindy_realis,
    //         cindy_account_id,
    //         alice_account_id.clone(),
    //         TokenId::from(42),
    //     );
    // }
    //
    // #[tokio::test]
    // async fn transfer_some_nfr_to_realis_than_use_them_3() {
    //     // Get Alice-realis account
    //     let (alice_account_id, private_alice_realis) =
    // accounts::realis::alice();     // Get Bob-realis account
    //     let (bob_account_id, private_bob_realis) = accounts::realis::bob();
    //     // Get Cindy-realis account
    //     let (cindy_account_id, private_cindy_realis) =
    // accounts::realis::cindy();     // Get Alice-bsc account
    //     let (address, private_bsc) = accounts::bsc::alice();
    //     // Alice-bsc transfer nft-51 to Alice-realis
    //     let _ = send_nft_from_bsc_to_realis(
    //         private_bsc,
    //         address,
    //         alice_account_id.clone(),
    //         TokenId::from(51),
    //     )
    //     .await;
    //     // Alice-bsc transfer nft-52 to Alice-realis
    //     let _ = send_nft_from_bsc_to_realis(
    //         private_bsc,
    //         address,
    //         alice_account_id.clone(),
    //         TokenId::from(52),
    //     )
    //     .await;
    //     // Alice-realis transfer nft-51 to Bob-realis
    //     transfer_nft_in_realis(
    //         private_alice_realis.clone(),
    //         alice_account_id.clone(),
    //         bob_account_id.clone(),
    //         TokenId::from(51),
    //     );
    //     // Alice-realis transfer nft-52 to Bob-realis
    //     transfer_nft_in_realis(
    //         private_alice_realis,
    //         alice_account_id.clone(),
    //         bob_account_id.clone(),
    //         TokenId::from(52),
    //     );
    //     // Bob-realis transfer nft-51 to Cindy-realis
    //     transfer_nft_in_realis(
    //         private_bob_realis.clone(),
    //         bob_account_id.clone(),
    //         cindy_account_id.clone(),
    //         TokenId::from(51),
    //     );
    //
    //     // Bob-realis transfer nft-51 to Alice-realis
    //     transfer_nft_in_realis(
    //         private_bob_realis,
    //         bob_account_id,
    //         alice_account_id.clone(),
    //         TokenId::from(52),
    //     );
    //     // Cindy-realis transfer nft-51 to Alice-realis
    //     transfer_nft_in_realis(
    //         private_cindy_realis,
    //         cindy_account_id,
    //         alice_account_id.clone(),
    //         TokenId::from(51),
    //     );
    // }
    //
    // #[tokio::test]
    // async fn two_way_token_from_realis() {
    //     // Get Alice-realis account
    //     let (account_id, private_realis) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (address, private_bsc) = accounts::bsc::alice();
    //     // Transfer 1 token from Alice-realis account to Alice-bsc account
    //     send_token_from_realis_to_bsc(
    //         private_realis,
    //         account_id.clone(),
    //         address,
    //         1,
    //     );
    //     // Transfer 1 token from Alice-bsc account to Alice-realis account
    //     send_tokens_from_bsc_to_realis(private_bsc, address, account_id,
    // 1).await; }
    //
    // #[tokio::test]
    // async fn two_way_token_from_bsc() {
    //     // Get Alice-realis account
    //     let (account_id, private_realis) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (address, private_bsc) = accounts::bsc::alice();
    //     // Transfer 1 token from Alice-bsc account to Alice-realis account
    //     send_tokens_from_bsc_to_realis(
    //         private_bsc,
    //         address,
    //         account_id.clone(),
    //         1,
    //     )
    //     .await;
    //     // Transfer 1 token from Alice-realis account to Alice-bsc account
    //     send_token_from_realis_to_bsc(
    //         private_realis,
    //         account_id.clone(),
    //         address,
    //         1,
    //     );
    // }
    //
    // #[tokio::test]
    // async fn two_way_nft_from_realis() {
    //     // Get Alice-realis account
    //     let (account_id, private_realis) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (address, private_bsc) = accounts::bsc::alice();
    //     // Transfer nft-61 from Alice-realis account to Alice-bsc account
    //     send_nft_from_realis_to_bsc(
    //         private_realis,
    //         account_id.clone(),
    //         address,
    //         TokenId::from(61),
    //     )
    //     .await;
    //     // Transfer nft-61 from Alice-bsc account to Alice-realis account
    //     send_nft_from_bsc_to_realis(
    //         private_bsc,
    //         address,
    //         account_id,
    //         TokenId::from(61),
    //     )
    //     .await;
    // }
    //
    // #[tokio::test]
    // async fn two_way_nft_from_bsc() {
    //     // Get Alice-realis account
    //     let (account_id, private_realis) = accounts::realis::alice();
    //     // Get Alice-bsc account
    //     let (address, private_bsc) = accounts::bsc::alice();
    //     // Transfer nft-62 from Alice-bsc account to Alice-realis account
    //     send_nft_from_bsc_to_realis(
    //         private_bsc,
    //         address,
    //         account_id.clone(),
    //         TokenId::from(62),
    //     )
    //     .await;
    //     // Transfer nft-62 from Alice-realis account to Alice-bsc account
    //     send_nft_from_realis_to_bsc(
    //         private_realis,
    //         account_id.clone(),
    //         address,
    //         TokenId::from(62),
    //     )
    //     .await;
    // }

    #[tokio::test]
    async fn test_01_transfer_tokens_to_bsc() {
        logger_setup();
        // Get stan client
        let stan_client = get_stan_client().await;
        // Get subject
        let subject = "realis-bridge".to_string();
        // Create request
        let request = serde_json::json!({
            "version": "test",
            "topic": "topic",
            "topic_res": "topic_res",
            "method": "transfer_token_to_bsc",
            "lang": "some_lang",
            "id": "01",
            "params": {
                "account_id": "5CSxbs1GPGgUZvsHNcFMyFRqu56jykBcBWBXhUBay2SXBsaA",
                "bsc_account": "0x6D1eee1CFeEAb71A4d7Fcc73f0EF67A9CA2cD943",
                "amount": "10000"
            },
        });
        // Publish request
        let request = stan_client
            .publish(subject, request.to_string().as_bytes())
            .await
            .unwrap();
        info!("Send {:?}", request);
        // Wait for response
        let response = get_response(&stan_client, String::from("response")).await;
        // Close connection
        let _ = stan_client.close().await;
        // Match result
        assert!(response.get("res").unwrap().get("req").is_some());
    }

    #[tokio::test]
    async fn test_02_transfer_nft_to_bsc() {
        logger_setup();
        // Get stan client
        let stan_client = get_stan_client().await;
        // Get subject
        let subject = "realis-bridge".to_string();
        // Create request
        let request = serde_json::json!({
            "id": "01",
            "method": "transfer_nft_to_bsc",
            "lang": "some_lang",
            "params": {
                "account_id": "5GgSgijLeCndfk1t8Mdjm8weUNEahBBtWwtfC1ZJxc9yNh1e",
                "bsc_account": "0x6D1eee1CFeEAb71A4d7Fcc73f0EF67A9CA2cD943",
                "token_id": "0x10000",
                "token_type": 3,
                "rarity": "Rare"
            },
            "agent": "agent",
            "authInfo": {
                "userId": "some user_id"
            },
        });
        // Publish request
        let request = stan_client
            .publish(subject, request.to_string().as_bytes())
            .await
            .unwrap();
        info!("Send {:?}", request);
        // Wait for response
        let response = get_response(&stan_client, String::from("response")).await;
        // Close connection
        let _ = stan_client.close().await;
        // Match result
        assert!(response.get("res").unwrap().get("req").is_some());
    }

    #[tokio::test]
    async fn test_03_transfer_tokens_to_realis() {
        logger_setup();
        // Get stan client
        let stan_client = get_stan_client().await;
        // Get subject
        let subject = "realis-bridge".to_string();
        // Create request
        let request = serde_json::json!({
            "id": "01",
            "method": "transfer_tokens_to_realis",
            "lang": "some_lang",
            "params": {
                "bsc_account": "0x6D1eee1CFeEAb71A4d7Fcc73f0EF67A9CA2cD943",
                "account_id": "5GgSgijLeCndfk1t8Mdjm8weUNEahBBtWwtfC1ZJxc9yNh1e",
                "amount": "10000"
            },
            "agent": "agent",
            "authInfo": {
                "userId": "some user_id"
            },
        });
        // Publish request
        let request = stan_client
            .publish(subject, request.to_string().as_bytes())
            .await
            .unwrap();
        info!("Send {:?}", request);
        // Wait for response
        let response = get_response(&stan_client, String::from("response")).await;
        // Close connection
        let _ = stan_client.close().await;
        // Match result
        assert!(response.get("res").unwrap().get("req").is_some());
    }

    #[tokio::test]
    async fn test_04_transfer_nft_to_realis() {
        logger_setup();
        // Get stan client
        let stan_client = get_stan_client().await;
        // Get subject
        let subject = "realis-bridge".to_string();
        // Create request
        let request = serde_json::json!({
            "id": "01",
            "method": "transfer_nft_to_realis",
            "lang": "some_lang",
            "params": {
                "bsc_account": "0x6D1eee1CFeEAb71A4d7Fcc73f0EF67A9CA2cD943",
                "account_id": "5GgSgijLeCndfk1t8Mdjm8weUNEahBBtWwtfC1ZJxc9yNh1e",
                "token_id": "0x10000",
                "token_type": 3,
                "rarity": "Rare"
            },
            "agent": "agent",
            "authInfo": {
                "userId": "some user_id"
            },
        });
        // Publish request
        let request = stan_client
            .publish(subject, request.to_string().as_bytes())
            .await
            .unwrap();
        info!("Send {:?}", request);
        // Wait for response
        let response = get_response(&stan_client, String::from("response")).await;
        // Close connection
        let _ = stan_client.close().await;
        // Match result
        assert!(response.get("res").unwrap().get("req").is_some());
    }

    // #[tokio::test]
    // async fn test_02_transfer_nft_to_bsc() {
    //     logger_setup();
    //     // Get stan client
    //     let stan_client = get_stan_client().await;
    //     // Get subject
    //     let subject = "realis-bridge";
    //     // Create request
    //     let request = serde_json::json!({
    //         "version": "test",
    //         "topic": "topic",
    //         "topic_res": "topic_res",
    //         "method": "transfer_nft_to_bsc",
    //         "lang": "some_lang",
    //         "id": "01",
    //         "params": {
    //         "account_id": "5Cfct15i277rkXX2Gxd58KgatVxAv2B4tmfrDPQRwvojFrUd",
    //             "bsc_account": "0x6D1eee1CFeEAb71A4d7Fcc73f0EF67A9CA2cD943",
    //             "token_id": TokenId::from(123),
    //             "token_type": 3,
    //             "rarity": "Common"
    //         },
    //     });
    //     // Publish request
    //     let request = stan_client
    //         .publish(subject, request.to_string().as_bytes())
    //         .await
    //         .unwrap();
    //     info!("Send {:?}", request);
    //     // Wait for response
    //     // let response = get_response(&stan_client,
    //     // String::from("response")).await; Close connection
    //     let _ = stan_client.close().await;
    //     // Match result
    //     // assert!(response.get("res").unwrap().get("req").is_some());
    // }
}
