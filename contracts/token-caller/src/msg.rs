use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
}

#[cw_serde]
pub enum ExecuteMsg {
    Transfer {
        contract: String, 
        recipient: String,
        amount: Uint128,
    },
    TransferFrom {
        contract: String, 
        owner: String, 
        recipient: String,
        amount: Uint128,
    },
    Approve {
        contract: String, 
        spender: String, 
        amount: Uint128,
        current_allowance: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
}
