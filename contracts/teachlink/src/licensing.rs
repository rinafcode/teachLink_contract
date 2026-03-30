pub enum LicenseType {
    Personal,
    Commercial,
    Exclusive,
    Subscription,
}

pub struct LicenseAgreement {
    pub token_id: u64,
    pub licensee: Address,
    pub license_type: LicenseType,
    pub expires_at: Option<u64>,
}

