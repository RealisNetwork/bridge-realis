mod accounts;

#[cfg(test)]
mod tests {
    use self::super::accounts;
    use runtime::AccountId;
    use substrate_api_client::{Api, compose_extrinsic, XtStatus};
    use substrate_api_client::sp_runtime::app_crypto::{sr25519, Pair};
    use web3::types::Address;

    fn api(signer: sr25519::Pair) -> Api<sr25519::Pair> {
        let url = "rpc.realis.network";
        // Create substrate api with signer
        Api::<sr25519::Pair>::new(format!("wss://{}", String::from(url)))
            .map(|api| api.set_signer(signer))
            .unwrap()
    }

    fn give_tokens(to: AccountId, amount: u128) {
        // Get sudo private key
        let (_, private) = accounts::realis::sudo();
        // Connect to Realis api
        let api = api(private);
        //
        let xt = compose_extrinsic!(
            api.clone(),
            "Balances",
            "transfer",
            to,
            amount
        );

        let tx_result =
            api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock);

        match tx_result {
            Ok(hash) => println!("Send extrinsic {:?}", hash),
            Err(error) => println!("Can`t send extrinsic {:?}", error),
        }
    }

    fn send_token_from_realis_to_bsc(signer: sr25519::Pair, from: AccountId, to: Address, amount: u128) {

    }

    #[tokio::test]
    async fn simple_token_transfer() {
        // Get Alice account
        let (account_id, private) = accounts::realis::alice();
        // Sudo give 1 token to A-realis
        give_tokens(account_id, 1000);
        // A-realis transfer 1 token to A-bsc
        // send_token_from_realis_to_bsc(private, account_id, )
        //
    }
}
