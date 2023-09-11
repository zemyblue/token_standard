use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{StdError, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Uint128,
}

impl InstantiateMsg {
    pub fn validate(&self) -> StdResult<()> {
        if !is_valid_name(&self.name) {
            return Err(StdError::generic_err(
                "Name is not in the expected format (3-50 UTF-8 bytes)",
            ));
        }
        if !is_valid_symbol(&self.symbol) {
            return Err(StdError::generic_err(
                "Ticker symbol is not in expected format [a-zA-Z\\-]{3,13}",
            ));
        }
        if self.decimals > 18 {
            return Err(StdError::generic_err("Decimals must not exceed 18"));
        }
        Ok(())
    }
}

fn is_valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.len() < 3 || bytes.len() > 12 {
        return false;
    }
    true
}

fn is_valid_symbol(symbol: &str) -> bool {
    let bytes = symbol.as_bytes();
    if bytes.len() < 3 || bytes.len() > 12 {
        return false;
    }
    for byte in bytes.iter() {
        if (*byte != 45) && (*byte < 65 || *byte > 90) && (*byte < 97 || *byte > 122) {
            return false;
        }
    }
    true
}

#[cw_serde]
pub enum ExecuteMsg {
    Transfer {
        recipient: String,
        amount: Uint128,
    },
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    Approve {
        spender: String,
        amount: Uint128,
        current_allowance: Uint128,
    },
    Receive {
        sender: String,
        amount: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(InfoResponse)]
    Info {},
    #[returns(TotalSupplyResponse)]
    TotalSupply {},
    #[returns(BalanceResponse)]
    Balance { owner: String },
    #[returns(AllowanceResponse)]
    Allowance { owner: String, spender: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct InfoResponse {
    pub name: String,
    pub symbol: String,
    pub decimal: u8,
    pub total_supply: Uint128,
}

// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[cw_serde]
pub struct TotalSupplyResponse {
    pub total_supply: Uint128,
}

// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[cw_serde]
pub struct BalanceResponse {
    pub balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct AllowanceResponse {
    pub allowance: Uint128,
}
