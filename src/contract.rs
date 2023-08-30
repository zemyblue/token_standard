#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Storage, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::event::{ApprovalEvent, Event, TransferEvent};
use crate::msg::{
    AllowanceResponse, ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg, TotalSupplyResponse,
};
use crate::state::{TokenInfo, ALLOWANCES, ALLOWANCES_SPENDER, BALANCES, TOKEN_INFO};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:token-standard";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // check valid token info
    msg.validate()?;

    let address = info.sender;
    BALANCES.save(deps.storage, &address, &msg.initial_balances)?;
    let total_supply = msg.initial_balances;

    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: total_supply,
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    Ok(Response::default())
    // Ok(Response::new()
    //     .add_attribute("method", "instantiate")
    //     .add_attribute("owner", info.sender)
    //     .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            handle_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => handle_transfer_from(deps, env, info, owner, recipient, amount),
        ExecuteMsg::Approve {
            spender,
            amount,
            current_allowance,
        } => handle_approve(deps, env, info, spender, amount, current_allowance),
    }
}

pub fn handle_transfer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    if amount == Uint128::zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let rcpt_addr = deps.api.addr_validate(&recipient)?;

    BALANCES.update(
        deps.storage,
        &info.sender,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_add(amount)?)
        },
    )?;

    // let res = Response::new()
    //     .add_attribute("action", "Transfer")
    //     .add_attribute("owner", info.sender)
    //     .add_attribute("recipient", recipient)
    //     .add_attribute("amount", amount);
    let mut rsp = Response::new();
    TransferEvent {
        owner: info.sender.as_ref(),
        recipient: &recipient.as_ref(),
        amount,
    }
    .add_attribute(&mut rsp);

    Ok(rsp)
}

pub fn handle_transfer_from(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: String,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let rcpt_addr = deps.api.addr_validate(&recipient)?;

    deduct_allowance(deps.storage, &owner_addr, &info.sender, amount)?;

    BALANCES.update(
        deps.storage,
        &owner_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        },
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_add(amount)?)
        },
    )?;

    let mut rsp = Response::new();
    TransferEvent {
        owner: &owner.as_str(),
        recipient: &recipient.as_ref(),
        amount,
    }
    .add_attribute(&mut rsp);

    Ok(rsp)
}

pub fn deduct_allowance(
    storage: &mut dyn Storage,
    owner: &Addr,
    spender: &Addr,
    amount: Uint128,
) -> Result<AllowanceResponse, ContractError> {
    let update_fn = |current: Option<AllowanceResponse>| -> _ {
        match current {
            Some(mut a) => {
                a.allowance = a
                    .allowance
                    .checked_sub(amount)
                    .map_err(StdError::overflow)?;
                Ok(a)
            }
            None => Err(ContractError::NoAllowance {}),
        }
    };
    ALLOWANCES.update(storage, (owner, spender), update_fn)?;
    ALLOWANCES_SPENDER.update(storage, (spender, owner), update_fn)
}

pub fn handle_approve(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    spender: String,
    amount: Uint128,
    current_allowance: Uint128,
) -> Result<Response, ContractError> {
    let spender_addr = deps.api.addr_validate(&spender)?;
    if spender_addr == info.sender {
        return Err(ContractError::CannotSetOwnAccount {});
    }

    let key = (&info.sender, &spender_addr);
    fn reverse<'a>(t: (&'a Addr, &'a Addr)) -> (&'a Addr, &'a Addr) {
        (t.1, t.0)
    }

    let old_allowance = ALLOWANCES.load(deps.storage, key)?;
    if current_allowance != old_allowance.allowance {
        return Err(ContractError::InvalidCurrentAllowance {});
    }
    let new_allowance = AllowanceResponse { allowance: amount };
    ALLOWANCES.save(deps.storage, key, &new_allowance)?;
    ALLOWANCES_SPENDER.save(deps.storage, reverse(key), &new_allowance)?;

    let mut rsp = Response::new();
    ApprovalEvent {
        owner: info.sender.as_ref(),
        spender: spender.as_ref(),
        old_amount: old_allowance.allowance,
        new_amount: amount,
    }
    .add_attribute(&mut rsp);

    Ok(rsp)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Info {} => to_binary(&query_info(deps)?),
        QueryMsg::TotalSupply {} => to_binary(&query_info(deps)?),
        QueryMsg::Allowance { owner, spender } => {
            to_binary(&query_allowance(deps, owner, spender)?)
        }
    }
}

pub fn query_info(deps: Deps) -> StdResult<InfoResponse> {
    let info = TOKEN_INFO.load(deps.storage)?;
    Ok(InfoResponse {
        name: info.name,
        symbol: info.symbol,
        decimal: info.decimals,
        total_supply: info.total_supply,
    })
}

pub fn query_total_supply(deps: Deps) -> StdResult<TotalSupplyResponse> {
    let info = TOKEN_INFO.load(deps.storage)?;
    Ok(TotalSupplyResponse {
        total_supply: info.total_supply,
    })
}

pub fn query_allowance(deps: Deps, owner: String, spender: String) -> StdResult<AllowanceResponse> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let spender_addr = deps.api.addr_validate(&spender)?;
    let allowance = ALLOWANCES
        .may_load(deps.storage, (&owner_addr, &spender_addr))?
        .unwrap_or_default();
    Ok(allowance)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
//     use cosmwasm_std::{coins, from_binary};

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(1000, "earth"));

//         // we can just call .unwrap() to assert this was a success
//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // it worked, let's query the state
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(17, value.count);
//     }

//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Increment {};
//         let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // should increase counter by 1
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }

//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let unauth_info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
//         match res {
//             Err(ContractError::Unauthorized {}) => {}
//             _ => panic!("Must return unauthorized error"),
//         }

//         // only the original creator can reset the counter
//         let auth_info = mock_info("creator", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

//         // should now be 5
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(5, value.count);
//     }
// }
