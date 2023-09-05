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
    use cosmwasm_std::{to_binary, SubMsg, Uint128, WasmMsg};

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

        let sub_msg = WasmMsg::Execute {
            contract_addr: contract,
            msg: to_binary(&TokenExecuteMsg::Transfer { recipient, amount })?,
            funds: vec![],
        };

        let rsp = Response::new().add_submessage(SubMsg::new(sub_msg));
        Ok(rsp)
    }

    pub fn transfer_from(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        contract: String,
        owner: String,
        recipient: String,
        amount: Uint128,
    ) -> Result<Response, ContractError> {
        if amount == Uint128::zero() {
            return Err(ContractError::InvalidZeroAmount {});
        }

        let sub_msg = WasmMsg::Execute {
            contract_addr: contract,
            msg: to_binary(&TokenExecuteMsg::TransferFrom {
                owner,
                recipient,
                amount,
            })?,
            funds: vec![],
        };

        Ok(Response::new().add_submessage(SubMsg::new(sub_msg)))
    }

    pub fn approve(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        contract: String,
        spender: String,
        amount: Uint128,
        current_allowance: Uint128,
    ) -> Result<Response, ContractError> {
        if amount == Uint128::zero() {
            return Err(ContractError::InvalidZeroAmount {});
        }

        let sub_msg = WasmMsg::Execute {
            contract_addr: contract,
            msg: to_binary(&TokenExecuteMsg::Approve {
                spender,
                amount,
                current_allowance,
            })?,
            funds: vec![],
        };

        Ok(Response::new().add_submessage(SubMsg::new(sub_msg)))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{mock_dependencies_with_balance, mock_env, mock_info},
        to_binary, SubMsg, Uint128, WasmMsg,
    };

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
        let recipient = String::from("recipient");
        let other_contract = String::from("contract");

        do_instantiate(deps.as_mut(), &creator);

        // transfer zero
        let transfer_amount = Uint128::new(10000);
        let msg = ExecuteMsg::Transfer {
            contract: other_contract.clone(),
            recipient: recipient.clone(),
            amount: transfer_amount,
        };
        let info = mock_info(creator.as_ref(), &[]);
        let env = mock_env();
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(1, res.messages.len());
        let expected = TokenExecuteMsg::Transfer {
            recipient,
            amount: transfer_amount,
        };
        assert_eq!(
            &res.messages[0],
            &SubMsg::new(WasmMsg::Execute {
                contract_addr: other_contract,
                msg: to_binary(&expected).unwrap(),
                funds: vec![]
            })
        )
    }
}
