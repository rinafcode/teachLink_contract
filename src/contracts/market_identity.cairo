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

    
    #[derive(Drop, starknet::Event)]
    struct DIDCreated {
        did_id: felt252,
        controller: ContractAddress,
        timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    struct DIDUpdated {
        did_id: felt252,
        old_controller: ContractAddress,
        new_controller: ContractAddress,
        timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    struct DIDDeactivated {
        did_id: felt252,
        controller: ContractAddress,
        timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    struct CredentialIssued {
        credential_id: felt252,
        issuer: ContractAddress,
        subject_did: felt252,
        credential_type: felt252,
        timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    struct CredentialRevoked {
        credential_id: felt252,
        issuer: ContractAddress,
        timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    struct AttestationCreated {
        attestation_id: felt252,
        attester: ContractAddress,
        credential_id: felt252,
        timestamp: u64,
    }

    #[derive(Drop, starknet::Event)]
    struct AttestationInvalidated {
        attestation_id: felt252,
        attester: ContractAddress,
        timestamp: u64,
    }

    
    #[derive(Drop, starknet::Event)]
    struct SelectiveDisclosureCreated {
        disclosure_id: felt252,
        credential_id: felt252,
        creator: ContractAddress,
        timestamp: u64,
    }

    #[constructor]
    fn constructor(ref self: ContractState, owner: ContractAddress) {
        self.contract_owner.write(owner);
        self.did_counter.write(0);
        self.credential_counter.write(0);
        self.attestation_counter.write(0);
        self.disclosure_counter.write(0);
        
        // Set owner as authorized issuer
        self.authorized_issuers.write(owner, true);
    }

    #[abi(embed_v0)]
    impl MarketXIdentityImpl of IMarketXIdentity<ContractState> {
        fn create_did(ref self: ContractState, controller: ContractAddress) -> felt252 {
            let caller = get_caller_address();
            assert(caller == controller, 'Only controller can create DID');
            
            let current_time = get_block_timestamp();
            let did_id = self.did_counter.read() + 1;
            self.did_counter.write(did_id);
            
            let new_did = DID {
                id: did_id,
                controller,
                created_at: current_time,
                updated_at: current_time,
                is_active: true,
            };
            
            self.dids.write(did_id, new_did);
            self.user_dids.write(controller, did_id);
            
            self.emit(DIDCreated {
                did_id,
                controller,
                timestamp: current_time,
            });
            
            did_id
        }


        fn update_did_controller(ref self: ContractState, did_id: felt252, new_controller: ContractAddress) {
            let caller = get_caller_address();
            let mut did = self.dids.read(did_id);
            
            assert(did.is_active, 'DID is not active');
            assert(caller == did.controller, 'Only controller can update');
            
            let old_controller = did.controller;
            did.controller = new_controller;
            did.updated_at = get_block_timestamp();
            
            self.dids.write(did_id, did);
            self.user_dids.write(new_controller, did_id);
            
            self.emit(DIDUpdated {
                did_id,
                old_controller,
                new_controller,
                timestamp: get_block_timestamp(),
            });
        }

        fn deactivate_did(ref self: ContractState, did_id: felt252) {
            let caller = get_caller_address();
            let mut did = self.dids.read(did_id);
            
            assert(did.is_active, 'DID already inactive');
            assert(caller == did.controller, 'Only controller can deactivate');
            
            did.is_active = false;
            did.updated_at = get_block_timestamp();
            
            self.dids.write(did_id, did);
            
            self.emit(DIDDeactivated {
                did_id,
                controller: did.controller,
                timestamp: get_block_timestamp(),
            });
        }

        fn get_did(self: @ContractState, did_id: felt252) -> DID {
            self.dids.read(did_id)
        }

