use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Cannot set to own account")]
    CannotSetOwnAccount {},

    #[error("Invalid zero amount")]
    InvalidZeroAmount {},

    #[error("Invalid current allowance")]
    InvalidCurrentAllowance {},

    #[error("No allowance for this account")]
    NoAllowance {},

    #[error("Recipient non-transferable")]
    NonTransferable {},
}
