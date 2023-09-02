use cosmwasm_std::{Event, Uint128};

pub fn transfer_event(owner: &str, recipient: &str, amount: Uint128) -> Event {
    Event::new("Transfer")
        .add_attribute("owner", owner.to_string())
        .add_attribute("recipient", recipient.to_string())
        .add_attribute("amount", amount.to_string())
}

pub fn approval_event(
    owner: &str,
    spender: &str,
    old_amount: Uint128,
    new_amount: Uint128,
) -> Event {
    Event::new("Approval")
        .add_attribute("owner", owner.to_string())
        .add_attribute("spender", spender.to_string())
        .add_attribute("old_amount", old_amount.to_string())
        .add_attribute("new_amount", new_amount.to_string())
}
