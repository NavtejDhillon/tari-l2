use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tari_l2_common::{Amount, Hash, PublicKey, Signature, crypto};
use crate::state::{ChannelState, Listing, Order, OrderStatus};
use tari_l2_common::{L2Error, error::Result};

/// State update operation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StateUpdate {
    /// Transfer funds between participants
    Transfer {
        from: PublicKey,
        to: PublicKey,
        amount: Amount,
    },

    /// Create a new listing
    CreateListing {
        listing: Listing,
    },

    /// Update listing status
    UpdateListing {
        listing_id: Hash,
        active: bool,
    },

    /// Create an order
    CreateOrder {
        order: Order,
    },

    /// Update order status
    UpdateOrderStatus {
        order_id: Hash,
        status: OrderStatus,
    },
}

impl StateUpdate {
    /// Apply this update to a state, returning the new state
    pub fn apply(&self, mut state: ChannelState) -> Result<ChannelState> {
        match self {
            StateUpdate::Transfer { from, to, amount } => {
                let from_balance = state.get_balance(from);
                let to_balance = state.get_balance(to);

                if from_balance < *amount {
                    return Err(L2Error::InsufficientBalance {
                        required: amount.value(),
                        available: from_balance.value(),
                    });
                }

                let new_from_balance = from_balance.checked_sub(*amount)
                    .ok_or(L2Error::InvalidStateTransition)?;
                let new_to_balance = to_balance.checked_add(*amount)
                    .ok_or(L2Error::InvalidStateTransition)?;

                state.set_balance(*from, new_from_balance);
                state.set_balance(*to, new_to_balance);
            }

            StateUpdate::CreateListing { listing } => {
                // Check if listing already exists
                if state.listings.iter().any(|l| l.id == listing.id) {
                    return Err(L2Error::InvalidStateTransition);
                }
                state.listings.push(listing.clone());
            }

            StateUpdate::UpdateListing { listing_id, active } => {
                let listing = state.listings.iter_mut()
                    .find(|l| &l.id == listing_id)
                    .ok_or(L2Error::InvalidStateTransition)?;
                listing.active = *active;
            }

            StateUpdate::CreateOrder { order } => {
                // Verify listing exists and is active
                let _listing = state.listings.iter()
                    .find(|l| l.id == order.listing_id && l.active)
                    .ok_or(L2Error::InvalidStateTransition)?;

                // Lock buyer's funds
                let buyer_balance = state.get_balance(&order.buyer);
                if buyer_balance < order.amount {
                    return Err(L2Error::InsufficientBalance {
                        required: order.amount.value(),
                        available: buyer_balance.value(),
                    });
                }

                state.orders.push(order.clone());
            }

            StateUpdate::UpdateOrderStatus { order_id, status } => {
                let order_idx = state.orders.iter()
                    .position(|o| &o.id == order_id)
                    .ok_or(L2Error::InvalidStateTransition)?;

                let order = &state.orders[order_idx];

                // On completion, transfer funds to seller
                if *status == OrderStatus::Completed && order.status != OrderStatus::Completed {
                    let buyer = order.buyer;
                    let seller = order.seller;
                    let amount = order.amount;

                    let buyer_balance = state.get_balance(&buyer);
                    let seller_balance = state.get_balance(&seller);

                    let new_buyer_balance = buyer_balance.checked_sub(amount)
                        .ok_or(L2Error::InvalidStateTransition)?;
                    let new_seller_balance = seller_balance.checked_add(amount)
                        .ok_or(L2Error::InvalidStateTransition)?;

                    state.set_balance(buyer, new_buyer_balance);
                    state.set_balance(seller, new_seller_balance);
                }

                state.orders[order_idx].status = status.clone();
            }
        }

        state.increment_nonce();
        Ok(state)
    }

    /// Calculate hash of this update for signing
    pub fn hash(&self) -> Hash {
        let serialized = bincode::serialize(self).expect("Serialization should not fail");
        crypto::hash_data(&serialized)
    }
}

/// Signed state update with all participant signatures
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedStateUpdate {
    pub update: StateUpdate,
    pub nonce: u64,
    pub signatures: HashMap<PublicKey, Signature>,
}

impl SignedStateUpdate {
    pub fn new(update: StateUpdate, nonce: u64) -> Self {
        Self {
            update,
            nonce,
            signatures: HashMap::new(),
        }
    }

    pub fn add_signature(&mut self, participant: PublicKey, signature: Signature) {
        self.signatures.insert(participant, signature);
    }

    /// Verify all signatures
    pub fn verify(&self, participants: &[PublicKey]) -> bool {
        // Must have signatures from all participants
        if self.signatures.len() != participants.len() {
            return false;
        }

        let message = self.signing_message();

        for participant in participants {
            match self.signatures.get(participant) {
                Some(sig) => {
                    if !crypto::verify_signature(participant, &message, sig) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }

    /// Get the message that should be signed
    fn signing_message(&self) -> Vec<u8> {
        let mut data = bincode::serialize(&self.update).expect("Serialization should not fail");
        data.extend_from_slice(&self.nonce.to_le_bytes());
        data
    }
}
