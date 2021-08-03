pub mod realis {
    use runtime::AccountId;
    use sp_core::crypto::Ss58Codec;
    use substrate_api_client::sp_runtime::app_crypto::{sr25519, Pair};

    pub fn sudo() -> (AccountId, sr25519::Pair) {
        let public =
            AccountId::from_ss58check("5CSxbs1GPGgUZvsHNcFMyFRqu56jykBcBWBXhUBay2SXBsaA")
                .unwrap();

        (public, private)
    }

    pub fn alice() -> (AccountId, sr25519::Pair) {
        let public = AccountId::from_ss58check(
            "5GKMzQnFtgEUUB8mLxXLvDkrj75AJJRaQs2XRTYgKgYQBdj1",
        )
        .unwrap();

        let private: sr25519::Pair = Pair::from_string(
            "wish dynamic depth wait depart column \
                farm abuse tail drink wear shallow",
            None,
        )
        .unwrap();

        (public, private)
    }

    pub fn bob() -> (AccountId, sr25519::Pair) {
        let public = AccountId::from_ss58check(
            "5F3uBTgjaUMWK7dZvTfiAjHw9y5teoUdCHfPziM7yoKMP5va",
        )
        .unwrap();

        let private: sr25519::Pair = Pair::from_string(
            "they first film either access soft \
                vanish boost finish hint identify bag",
            None,
        )
        .unwrap();

        (public, private)
    }

    pub fn cindy() -> (AccountId, sr25519::Pair) {
        let public = AccountId::from_ss58check(
            "5Grk9Ckfp6bunGUYzZXQEYabZkqa6WRkQYrQaKTPDxvAQ9JB",
        )
        .unwrap();

        let private: sr25519::Pair = Pair::from_string(
            "industry trash cause horse kangaroo \
                fiscal obey someone fortune shrimp wrestle fatigue",
            None,
        )
        .unwrap();

        (public, private)
    }
}

pub mod bsc {
    use secp256k1::SecretKey;
    use std::str::FromStr;
    use web3::types::Address;

    pub fn sudo() -> (Address, SecretKey) {
        let public =
            Address::from_str("0x79abf92F6640B6D6540B116d4e7eA99ace932236")
                .unwrap();

        let private =
            SecretKey::from_str(
                ""
            ).unwrap();

        (public, private)
    }

    pub fn alice() -> (Address, SecretKey) {
        let public =
            Address::from_str("0xF09c06d1cDD0cF2B8Cc691197Eb1dd2d6889b5b1")
                .unwrap();

        let private = SecretKey::from_str(
            "f807147ea8a83e0951a23cbd2ff5c3a9cbaff5ca49234bd7b9d73b052b6fccb9",
        )
        .unwrap();

        (public, private)
    }

    pub fn bob() -> (Address, SecretKey) {
        let public =
            Address::from_str("0x75b882a65e82C00924a83e93bC39584C2Bb7a39B")
                .unwrap();

        let private = SecretKey::from_str(
            "2e7738fb3006d77faa35217a0f9fd7c03b19db0e717d4b909668b413d7b9b54f",
        )
        .unwrap();

        (public, private)
    }

    pub fn cindy() -> (Address, SecretKey) {
        let public =
            Address::from_str("0x6dFF4Fab42659d677c4c60F5FCC6993A96b5aaEa")
                .unwrap();

        let private = SecretKey::from_str(
            "f2d7a63c6aa4e22ade393093a5be78ec4f012c40d785b212aa2884e95f1b4b67",
        )
        .unwrap();

        (public, private)
    }
}
