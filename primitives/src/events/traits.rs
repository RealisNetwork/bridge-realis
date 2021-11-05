use runtime::Call;

use ethabi::token::Token;

pub trait Event {
    fn get_realis_call(&self) -> Call;

    fn get_binance_call(&self) -> (String, (Token, Token, Token));
}