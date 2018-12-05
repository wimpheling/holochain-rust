use crate::{
    action::{Action, ActionWrapper},
    context::Context,
    instance::dispatch_action,
    network::direct_message::DirectMessage,
    nucleus::actions::build_validation_package::build_validation_package,
};

use holochain_core_types::{cas::content::Address, entry::Entry, error::HolochainError};
use std::{convert::TryFrom, sync::Arc};

fn get_entry(address: &Address, context: &Arc<Context>) -> Result<Entry, HolochainError> {
    let raw = context
        .state()
        .unwrap()
        .agent()
        .chain()
        .content_storage()
        .read()
        .unwrap()
        .fetch(address)?
        .ok_or(HolochainError::ErrorGeneric("Entry not found".to_string()))?;

    Entry::try_from(raw)
}

pub async fn respond_validation_package_request(
    to_agent_id: Address,
    msg_id: String,
    requested_entry_address: Address,
    context: Arc<Context>,
) {
    let maybe_validation_package = match get_entry(&requested_entry_address, &context) {
        Ok(entry) => await!(build_validation_package(&entry, &context)).ok(),
        Err(_) => None,
    };

    let direct_message = DirectMessage::ValidationPackage(maybe_validation_package);
    let action_wrapper = ActionWrapper::new(Action::SendDirectMessage((
        to_agent_id,
        direct_message,
        msg_id,
        true,
    )));
    dispatch_action(&context.action_channel, action_wrapper);
}