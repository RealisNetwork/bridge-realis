use realis_primitives::{Basic, TokenId};
use runtime::AccountId;
use sp_core::H160;

pub enum Events {
    TokenRealisToBsc(AccountId, H160, u128),
    NftRealisToBsc(AccountId, H160, TokenId, Basic),

    TokenSuccessOnBsc(AccountId, u128),
    NftSuccessOnBsc(AccountId, TokenId, Basic),

    TokenErrorOnBsc(AccountId, H160, u128),
    NftErrorOnBsc(AccountId, H160, TokenId, Basic),

    TokenBscToRealis(AccountId, u128),
    NftBcsToRealis(AccountId, TokenId, Basic),

    TokenSuccessOnRealis(AccountId, u128),
    NftSuccessOnRealis(AccountId, TokenId),

    TokenErrorOnRealis(AccountId, u128),
    NftErrorOnRealis(AccountId, TokenId),
}
