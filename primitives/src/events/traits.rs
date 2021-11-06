use runtime::Call;

use ethabi::token::Token;

/// Common event for both Realis and Binance blockchains:
/// 1. Event emits on Realis blockchain
/// In this case user want to transfer some staff from Realis to Binance.
/// That means user already transfer his staff (from his account to Realis-Bridge account) on Realis side,
/// so we should transfer on Binance side (from Binance-Bridge(contract) account to user account).
/// Because of that `get_binance_call` is main call for Binance blockchain to do that transfer
/// and `get_realis_call` is a rollback call for Realis blockchain.
/// If Binance call fail we should return state on Realis blockchain
/// 2. Event emits on Binance blockchain
/// This case has same logic with one difference.
/// In this case `get_realis_call` is main call for Realis blockchain
/// and `get_binance_call` is a rollback call if Realis blockchain call fail
pub trait Event {
    fn get_hash(&self) -> String;

    fn get_realis_call(&self) -> Call;

    fn get_binance_call(&self) -> (String, (Token, Token, Token));
}
