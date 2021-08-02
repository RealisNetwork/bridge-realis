#[cfg(test)]
mod accounts;

#[cfg(test)]
mod tests {
    use self::super::accounts;
    use runtime::{realis_bridge::Call as RealisBridgeCall, AccountId, Call};
    use secp256k1::SecretKey;
    use sp_core::{H160, H256 as Hash};
    use sp_runtime::{generic, traits::BlakeTwo256};
    use substrate_api_client::{
        compose_extrinsic_offline,
        sp_runtime::app_crypto::{sr25519, Pair},
        Api, BlockNumber, XtStatus,
    };
    use tokio::time::{sleep, Duration};
    use utils;
    use web3::types::Address;

    type Header = generic::Header<BlockNumber, BlakeTwo256>;

    fn api(signer: sr25519::Pair) -> Api<sr25519::Pair> {
        // Url to realis blockchain
        let url = "rpc.realis.network";
        // Create substrate api with signer
        Api::<sr25519::Pair>::new(format!("wss://{}", String::from(url)))
            .map(|api| api.set_signer(signer))
            .unwrap()
    }

    fn send_token_from_realis_to_bsc(
        signer: sr25519::Pair,
        from: AccountId,
        to: Address,
        amount: u128,
    ) {
        let api = api(signer);
        // Create some parameters for transaction
        let head: Hash = api.get_finalized_head().unwrap().unwrap();
        let h: Header = api.get_header(Some(head)).unwrap().unwrap();
        let period = 5;
        // Create transaction
        #[allow(clippy::redundant_clone)]
        let xt = compose_extrinsic_offline!(
            api.clone().signer.unwrap(),
            Call::RealisBridge(RealisBridgeCall::transfer_token_to_bsc(
                from.clone(),
                H160::from(to.0),
                amount * 10_000_000_000
            )),
            api.get_nonce().unwrap(),
            Era::mortal(period, h.number),
            api.genesis_hash,
            head,
            api.runtime_version.spec_version,
            api.runtime_version.transaction_version
        );
        // Send extrinsic transaction
        let tx_result = api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);
    }

    async fn transfer_token_in_bsc(
        signer: SecretKey,
        _from: Address,
        to: Address,
        amount: u128,
    ) {
        // Connect to bsc smart contract
        let contract = utils::contract::token_new().await;
        // Send transaction
        contract
            .signed_call_with_confirmations(
                "transfer",
                (to, amount * 10_000_000_000),
                web3::contract::Options::default(),
                1,
                &signer,
            )
            .await;
    }

    #[test]
    fn simple_token_transfer() {
        // Get Alice-realis account
        let (account_id, private) = accounts::realis::alice();
        // Get Alice-bsc account
        let (address, _) = accounts::bsc::alice();
        // A-realis transfer 1 token to A-bsc
        send_token_from_realis_to_bsc(private, account_id, address, 1);
    }

    #[test]
    fn transfer_some_tokens() {
        // Get Alice-realis account
        let (account_id, private) = accounts::realis::alice();
        // Get Alice-bsc account
        let (address, _) = accounts::bsc::alice();
        // A-realis transfer 1000 tokens to A-bsc
        send_token_from_realis_to_bsc(private, account_id, address, 1000);
    }

    #[tokio::test]
    async fn transfer_some_tokens_than_use_them_1() {
        // Get Alice-realis account
        let (account_id, private_realis) = accounts::realis::alice();
        // Get Alice-bsc account
        let (alice_address, private_bsc) = accounts::bsc::alice();
        // Get Bob-bsc account
        let (bob_address, _) = accounts::bsc::bob();
        // A-realis transfer 1000 tokens to A-bsc
        send_token_from_realis_to_bsc(
            private_realis,
            account_id,
            alice_address,
            1000,
        );
        //
        sleep(Duration::from_millis(10_000)).await;
        // A-bsc transfer 1000 tokens to B-bsc
        transfer_token_in_bsc(private_bsc, alice_address, bob_address, 1000)
            .await;
    }

    #[tokio::test]
    async fn transfer_some_tokens_than_use_them_2() {
        // Get Alice-realis account
        let (account_id, private_realis) = accounts::realis::alice();
        // Get Alice-bsc account
        let (alice_address, private_bsc) = accounts::bsc::alice();
        // Get Bob-bsc account
        let (bob_address, _) = accounts::bsc::bob();
        // Get Cindy-bsc account
        let (cindy_address, _) = accounts::bsc::cindy();
        // A-realis transfer 1000 tokens to A-bsc
        send_token_from_realis_to_bsc(
            private_realis,
            account_id,
            alice_address,
            1000,
        );
        // Wait for transaction end
        sleep(Duration::from_millis(10_000)).await;
        // A-bsc transfer 300 tokens to B-bsc
        transfer_token_in_bsc(private_bsc, alice_address, bob_address, 300)
            .await;
        // Wait for transaction end
        sleep(Duration::from_millis(10_000)).await;
        // A-bsc transfer 300 tokens to C-bsc
        transfer_token_in_bsc(private_bsc, alice_address, cindy_address, 300)
            .await;
    }

    #[tokio::test]
    async fn transfer_some_tokens_than_use_them_3() {
        // Get Alice-realis account
        let (account_id, private_realis) = accounts::realis::alice();
        // Get Alice-bsc account
        let (alice_address, private_alice_bsc) = accounts::bsc::alice();
        // Get Bob-bsc account
        let (bob_address, private_bob_bsc) = accounts::bsc::bob();
        // Get Cindy-bsc account
        let (cindy_address, _) = accounts::bsc::cindy();
        // A-realis transfer 1000 tokens to A-bsc
        send_token_from_realis_to_bsc(
            private_realis,
            account_id,
            alice_address,
            1000,
        );
        // Wait for transaction end
        sleep(Duration::from_millis(10_000)).await;
        // A-bsc transfer 700 tokens to B-bsc
        transfer_token_in_bsc(
            private_alice_bsc,
            alice_address,
            bob_address,
            700,
        )
        .await;
        // Wait for transaction end
        sleep(Duration::from_millis(10_000)).await;
        // A-bsc transfer 400 tokens to C-bsc
        transfer_token_in_bsc(private_bob_bsc, bob_address, cindy_address, 400)
            .await;
    }
}
