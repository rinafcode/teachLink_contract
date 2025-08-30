use starknet::ContractAddress;

#[derive(Drop, Serde, starknet::Store)]
struct DID {
    id: felt252,
    controller: ContractAddress,
    created_at: u64,
    updated_at: u64,
    is_active: bool,
}

#[derive(Drop, Serde, starknet::Store)]
struct VerifiableCredential {
    id: felt252,
    issuer: ContractAddress,
    subject: felt252, // DID of the subject
    credential_type: felt252,
    achievement_data: felt252, // Hash of achievement data
    issued_at: u64,
    expires_at: u64,
    is_revoked: bool,
}

#[derive(Drop, Serde, starknet::Store)]
struct Attestation {
    id: felt252,
    attester: ContractAddress,
    credential_id: felt252,
    attestation_data: felt252,
    created_at: u64,
    is_valid: bool,
}

#[derive(Drop, Serde, starknet::Store)]
struct SelectiveDisclosure {
    credential_id: felt252,
    disclosed_fields: Array<felt252>,
    proof_hash: felt252,
    created_at: u64,
}
