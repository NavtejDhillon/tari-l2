use serde::{Deserialize, Serialize};
use tari_l2_common::{Amount, Hash, PublicKey, Timestamp};

/// Escrow contract status
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EscrowStatus {
    /// Escrow created, waiting for buyer to fund
    Created,
    /// Buyer has funded the escrow
    Funded,
    /// Seller has marked as shipped/delivered
    Shipped,
    /// Buyer has confirmed receipt, funds released to seller
    Completed,
    /// Buyer requested refund
    RefundRequested,
    /// Refund processed, funds returned to buyer
    Refunded,
    /// Dispute raised, needs arbitration
    Disputed,
    /// Cancelled before funding
    Cancelled,
}

/// Escrow contract for a marketplace transaction
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EscrowContract {
    /// Unique escrow ID
    pub id: Hash,

    /// The listing being purchased
    pub listing_id: Hash,

    /// Buyer's public key
    pub buyer: PublicKey,

    /// Seller's public key
    pub seller: PublicKey,

    /// Amount held in escrow
    pub amount: Amount,

    /// Current status
    pub status: EscrowStatus,

    /// When the escrow was created
    pub created_at: Timestamp,

    /// When the escrow was last updated
    pub updated_at: Timestamp,

    /// Timeout period (seconds) - auto-release to seller if buyer doesn't confirm
    pub timeout_period: u64,

    /// Transaction ID on L1 (if funds locked on-chain)
    pub l1_tx_id: Option<String>,

    /// Shipping/tracking information
    pub tracking_info: Option<String>,

    /// Dispute reason (if any)
    pub dispute_reason: Option<String>,
}

impl EscrowContract {
    pub fn new(
        listing_id: Hash,
        buyer: PublicKey,
        seller: PublicKey,
        amount: Amount,
        timeout_period: u64,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id: Hash::random(),
            listing_id,
            buyer,
            seller,
            amount,
            status: EscrowStatus::Created,
            created_at: now,
            updated_at: now,
            timeout_period,
            l1_tx_id: None,
            tracking_info: None,
            dispute_reason: None,
        }
    }

    /// Check if escrow has timed out (auto-release condition)
    pub fn is_timed_out(&self) -> bool {
        if self.status != EscrowStatus::Shipped {
            return false;
        }

        let elapsed = Timestamp::now().as_secs() - self.updated_at.as_secs();
        elapsed > self.timeout_period
    }

    /// Fund the escrow (buyer deposits funds)
    pub fn fund(&mut self, l1_tx_id: String) -> Result<(), String> {
        if self.status != EscrowStatus::Created {
            return Err(format!("Cannot fund escrow in status {:?}", self.status));
        }

        self.status = EscrowStatus::Funded;
        self.l1_tx_id = Some(l1_tx_id);
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Mark as shipped (seller)
    pub fn mark_shipped(&mut self, tracking_info: Option<String>) -> Result<(), String> {
        if self.status != EscrowStatus::Funded {
            return Err(format!("Cannot ship from status {:?}", self.status));
        }

        self.status = EscrowStatus::Shipped;
        self.tracking_info = tracking_info;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Confirm receipt and release funds (buyer)
    pub fn confirm_receipt(&mut self) -> Result<(), String> {
        if self.status != EscrowStatus::Shipped {
            return Err(format!("Cannot confirm from status {:?}", self.status));
        }

        self.status = EscrowStatus::Completed;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Request refund (buyer)
    pub fn request_refund(&mut self, reason: String) -> Result<(), String> {
        if self.status != EscrowStatus::Funded && self.status != EscrowStatus::Shipped {
            return Err(format!("Cannot refund from status {:?}", self.status));
        }

        self.status = EscrowStatus::RefundRequested;
        self.dispute_reason = Some(reason);
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Approve refund (seller agrees)
    pub fn approve_refund(&mut self) -> Result<(), String> {
        if self.status != EscrowStatus::RefundRequested {
            return Err(format!("Cannot refund from status {:?}", self.status));
        }

        self.status = EscrowStatus::Refunded;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Raise dispute
    pub fn raise_dispute(&mut self, reason: String) -> Result<(), String> {
        if self.status == EscrowStatus::Completed ||
           self.status == EscrowStatus::Refunded ||
           self.status == EscrowStatus::Cancelled {
            return Err(format!("Cannot dispute from status {:?}", self.status));
        }

        self.status = EscrowStatus::Disputed;
        self.dispute_reason = Some(reason);
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Auto-release to seller after timeout
    pub fn auto_release(&mut self) -> Result<(), String> {
        if !self.is_timed_out() {
            return Err("Escrow has not timed out yet".to_string());
        }

        self.status = EscrowStatus::Completed;
        self.updated_at = Timestamp::now();
        Ok(())
    }
}
