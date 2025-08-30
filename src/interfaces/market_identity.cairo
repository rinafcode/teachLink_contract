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

#[starknet::interface]
trait IMarketXIdentity<TContractState> {
    // DID Management
    fn create_did(ref self: TContractState, controller: ContractAddress) -> felt252;
    fn update_did_controller(ref self: TContractState, did_id: felt252, new_controller: ContractAddress);
    fn deactivate_did(ref self: TContractState, did_id: felt252);
    fn get_did(self: @TContractState, did_id: felt252) -> DID;
    
    // Credential Management
    fn issue_credential(
        ref self: TContractState,
        subject_did: felt252,
        credential_type: felt252,
        achievement_data: felt252,
        expires_at: u64
    ) -> felt252;
    fn revoke_credential(ref self: TContractState, credential_id: felt252);
    fn get_credential(self: @TContractState, credential_id: felt252) -> VerifiableCredential;
    fn verify_credential(self: @TContractState, credential_id: felt252) -> bool;
    
    // Selective Disclosure
    fn create_selective_disclosure(
        ref self: TContractState,
        credential_id: felt252,
        disclosed_fields: Array<felt252>,
        proof_hash: felt252
    ) -> felt252;
    fn verify_selective_disclosure(
        self: @TContractState,
        disclosure_id: felt252,
        proof: felt252
    ) -> bool;
    
    // Attestation System
    fn create_attestation(
        ref self: TContractState,
        credential_id: felt252,
        attestation_data: felt252
    ) -> felt252;
    fn verify_attestation(self: @TContractState, attestation_id: felt252) -> bool;
    fn invalidate_attestation(ref self: TContractState, attestation_id: felt252);
    
    // Query Functions
    fn get_user_credentials(self: @TContractState, did_id: felt252) -> Array<felt252>;
    fn get_credential_attestations(self: @TContractState, credential_id: felt252) -> Array<felt252>;
    fn is_credential_valid(self: @TContractState, credential_id: felt252) -> bool;
}
