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
    AllowanceResponse, BalanceResponse, ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg,
    TotalSupplyResponse,
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
        total_supply,
    };
    TOKEN_INFO.save(deps.storage, &data)?;

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

    let mut rsp = Response::new();
    TransferEvent {
        owner: info.sender.as_ref(),
        recipient: recipient.as_ref(),
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
        owner: owner.as_str(),
        recipient: recipient.as_ref(),
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

    let old_allowance = ALLOWANCES.may_load(deps.storage, key)?.unwrap_or_default();
    if current_allowance != old_allowance.allowance {
        return Err(ContractError::InvalidCurrentAllowance {});
    }
    if amount == Uint128::zero() {
        ALLOWANCES.remove(deps.storage, key);
        ALLOWANCES_SPENDER.remove(deps.storage, reverse(key));
    } else {
        let new_allowance = AllowanceResponse { allowance: amount };
        ALLOWANCES.save(deps.storage, key, &new_allowance)?;
        ALLOWANCES_SPENDER.save(deps.storage, reverse(key), &new_allowance)?;
    }

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
        QueryMsg::Balance { owner } => to_binary(&query_balance(deps, owner)?),
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

pub fn query_balance(deps: Deps, owner: String) -> StdResult<BalanceResponse> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let balance = BALANCES
        .may_load(deps.storage, &owner_addr)?
        .unwrap_or_default();
    Ok(BalanceResponse { balance })
}

pub fn query_allowance(deps: Deps, owner: String, spender: String) -> StdResult<AllowanceResponse> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let spender_addr = deps.api.addr_validate(&spender)?;
    let allowance = ALLOWANCES
        .may_load(deps.storage, (&owner_addr, &spender_addr))?
        .unwrap_or_default();
    Ok(allowance)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::msg::InstantiateMsg;
    use cosmwasm_std::{
        attr,
        testing::{mock_dependencies_with_balance, mock_env, mock_info},
    };

    fn do_instantiate(mut deps: DepsMut, creator: &str, amount: Uint128) -> InfoResponse {
        let instantiate_msg = InstantiateMsg {
            name: "Test".to_string(),
            symbol: "TST".to_string(),
            decimals: 8,
            initial_balances: amount,
        };
        let info = mock_info(creator, &[]);
        let env = mock_env();
        let res = instantiate(deps.branch(), env, info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());
        query_info(deps.as_ref()).unwrap()
    }

    #[test]
    fn transfer() {
        let mut deps = mock_dependencies_with_balance(&[]);
        let creator = String::from("creator");
        let recipient = String::from("recipient");

        let init_balance = Uint128::new(1000000000);
        do_instantiate(deps.as_mut(), &creator, init_balance);

        // transfer zero
        let msg = ExecuteMsg::Transfer {
            recipient: recipient.clone(),
            amount: Uint128::zero(),
        };
        let info = mock_info(creator.as_ref(), &[]);
        let env = mock_env();
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(err, ContractError::InvalidZeroAmount {});

        // transfer normal
        let amount = Uint128::new(1000);
        let msg = ExecuteMsg::Transfer {
            recipient: recipient.clone(),
            amount,
        };
        let info = mock_info(creator.as_ref(), &[]);
        let env = mock_env();
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.attributes[0], attr("action", "Transfer"));
    }

    #[test]
    fn transfer_from() {
        let mut deps = mock_dependencies_with_balance(&[]);
        let owner = String::from("owner");
        let spender = String::from("spender");
        let recipient = String::from("recipient");

        let init_balance = Uint128::new(1000000000);
        do_instantiate(deps.as_mut(), &owner, init_balance);

        // approve
        let allow1 = Uint128::new(1000000);
        let msg = ExecuteMsg::Approve {
            spender: spender.clone(),
            amount: allow1,
            current_allowance: Uint128::zero(),
        };
        let info = mock_info(owner.as_ref(), &[]);
        let env = mock_env();
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.attributes[0], attr("action", "Approval"));
        let allowance = query_allowance(deps.as_ref(), owner.clone(), spender.clone()).unwrap();
        assert_eq!(allow1, allowance.allowance);

        // transfer_from
        let amount = Uint128::new(500000);
        let msg = ExecuteMsg::TransferFrom {
            owner: owner.clone(),
            recipient: recipient.clone(),
            amount: amount,
        };
        let info = mock_info(spender.as_ref(), &[]);
        let env = mock_env();
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.attributes[0], attr("action", "Transfer"));
        let res = query_balance(deps.as_ref(), owner.clone()).unwrap();
        assert_eq!(res.balance, init_balance - amount);
    }
}
