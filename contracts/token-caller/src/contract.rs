use cosmwasm_std::{entry_point, Binary, Deps, StdResult};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

#[cfg(not(feature = "library"))]
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use token_standard::ExecuteMsg as TokenExecuteMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:token-caller";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer {
            contract,
            recipient,
            amount,
        } => exec::transfer(deps, env, info, contract, recipient, amount),
        ExecuteMsg::TransferFrom {
            contract,
            owner,
            recipient,
            amount,
        } => exec::transfer_from(deps, env, info, contract, owner, recipient, amount),
        ExecuteMsg::Approve {
            contract,
            spender,
            amount,
            current_allowance,
        } => exec::approve(
            deps,
            env,
            info,
            contract,
            spender,
            amount,
            current_allowance,
        ),
    }
}

mod exec {
    use cosmwasm_std::{to_binary, Uint128, WasmMsg};

    use super::*;

    pub fn transfer(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        contract: String,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        if amount == Uint128::zero() {
            return Err(ContractError::InvalidZeroAmount {});
        }

        let sub_transfer_msg = TokenExecuteMsg::Transfer { recipient, amount: amount.into() };
        let sub_msg = WasmMsg::Execute {
            contract_addr: contract,
            msg: to_binary(&sub_transfer_msg)?,
            funds: vec![],
        };

        let rsp = Response::new().add_message(sub_msg);
        Ok(rsp)
    }

    pub fn transfer_from(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _contract: String,
        _owner: String,
        _recipient: String,
        _amount: Uint128,
    ) -> Result<Response, ContractError> {
        Ok(Response::default())
    }

    pub fn approve(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _contract: String,
        _spender: String,
        _amount: Uint128,
        _current_allowance: Uint128,
    ) -> Result<Response, ContractError> {
        Ok(Response::default())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_info, mock_env, mock_dependencies_with_balance};

    use super::*;

    fn do_instantiate(mut deps: DepsMut, creator: &str) {
        let instantiate_msg = InstantiateMsg {};
        let info: MessageInfo = mock_info(creator, &[]);
        let env = mock_env();
        let res = instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn transfer() {
        let mut deps = mock_dependencies_with_balance(&[]);
        let creator = String::from("creator");
        let _recipient = String::from("recipient");

        do_instantiate(deps.as_mut(), &creator)

        // transfer zero

    }
}
