#[starknet::contract]
mod MarketXIdentity {
    use starknet::{ContractAddress, get_caller_address, get_block_timestamp};
    use super::interfaces::IMarketXIdentity::{
        IMarketXIdentity, DID, VerifiableCredential, Attestation, SelectiveDisclosure
    };
    use super::libraries::CredentialVerification::CredentialVerificationTrait;
    use hash::HashTrait;
    use pedersen::PedersenTrait;

    #[storage]
    struct Storage {
        // DID storage
        dids: LegacyMap<felt252, DID>,
        did_counter: felt252,
        user_dids: LegacyMap<ContractAddress, felt252>,
        
        // Credential storage
        credentials: LegacyMap<felt252, VerifiableCredential>,
        credential_counter: felt252,
        user_credentials: LegacyMap<felt252, Array<felt252>>, // DID -> credential IDs
        revoked_credentials: LegacyMap<felt252, bool>,
        
        // Attestation storage
        attestations: LegacyMap<felt252, Attestation>,
        attestation_counter: felt252,
        credential_attestations: LegacyMap<felt252, Array<felt252>>, // credential_id -> attestation IDs
        
        // Selective disclosure storage
        selective_disclosures: LegacyMap<felt252, SelectiveDisclosure>,
        disclosure_counter: felt252,
        
        // Access control
        authorized_issuers: LegacyMap<ContractAddress, bool>,
        contract_owner: ContractAddress,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        DIDCreated: DIDCreated,
        DIDUpdated: DIDUpdated,
        DIDDeactivated: DIDDeactivated,
        CredentialIssued: CredentialIssued,
        CredentialRevoked: CredentialRevoked,
        AttestationCreated: AttestationCreated,
        AttestationInvalidated: AttestationInvalidated,
        SelectiveDisclosureCreated: SelectiveDisclosureCreated,
    }
