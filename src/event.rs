use cosmwasm_std::{attr, Response, Uint128};

pub trait Event {
    /// Append attributes to response
    fn add_attribute(&self, response: &mut Response);
}

// Transfer Event
pub struct TransferEvent<'a> {
    pub owner: &'a str,
    pub recipient: &'a str,
    pub amount: Uint128,
}

impl<'a> Event for TransferEvent<'a> {
    fn add_attribute(&self, rsp: &mut Response) {
        rsp.attributes.push(attr("action", "Transfer"));
        rsp.attributes.push(attr("owner", self.owner.to_string()));
        rsp.attributes.push(attr("recipient", self.recipient.to_string()));
        rsp.attributes.push(attr("amount", self.amount.to_string()));
    }
}

// Approval Events
pub struct ApprovalEvent<'a> {
    pub owner: &'a str, 
    pub spender: &'a str,
    pub old_amount: Uint128,
    pub new_amount: Uint128,
}

impl<'a> Event for ApprovalEvent<'a> {
    fn add_attribute(&self, rsp: &mut Response) {
        rsp.attributes.push(attr("action", "Approval"));
        rsp.attributes.push(attr("owner", self.owner.to_string()));
        rsp.attributes.push(attr("spender", self.spender.to_string()));
        rsp.attributes.push(attr("old_amount", self.old_amount.to_string()));
        rsp.attributes.push(attr("new_amount", self.new_amount.to_string()));
    }
}
