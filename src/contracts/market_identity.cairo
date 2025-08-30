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


        fn issue_credential(
            ref self: ContractState,
            subject_did: felt252,
            credential_type: felt252,
            achievement_data: felt252,
            expires_at: u64
        ) -> felt252 {
            let caller = get_caller_address();
            assert(self.authorized_issuers.read(caller), 'Not authorized issuer');
            
            let subject = self.dids.read(subject_did);
            assert(subject.is_active, 'Subject DID not active');
            
            let current_time = get_block_timestamp();
            assert(expires_at > current_time, 'Invalid expiry time');
            
            let credential_id = self.credential_counter.read() + 1;
            self.credential_counter.write(credential_id);
            
            let new_credential = VerifiableCredential {
                id: credential_id,
                issuer: caller,
                subject: subject_did,
                credential_type,
                achievement_data,
                issued_at: current_time,
                expires_at,
                is_revoked: false,
            };
            
            self.credentials.write(credential_id, new_credential);
            
            // Add to user's credentials
            let mut user_creds = self.user_credentials.read(subject_did);
            user_creds.append(credential_id);
            self.user_credentials.write(subject_did, user_creds);
            
            self.emit(CredentialIssued {
                credential_id,
                issuer: caller,
                subject_did,
                credential_type,
                timestamp: current_time,
            });
            
            credential_id
        }

        fn revoke_credential(ref self: ContractState, credential_id: felt252) {
            let caller = get_caller_address();
            let credential = self.credentials.read(credential_id);

            
            assert(caller == credential.issuer, 'Only issuer can revoke');
            assert(!credential.is_revoked, 'Already revoked');
            
            self.revoked_credentials.write(credential_id, true);
            
            self.emit(CredentialRevoked {
                credential_id,
                issuer: caller,
                timestamp: get_block_timestamp(),
            });
        }

        fn get_credential(self: @ContractState, credential_id: felt252) -> VerifiableCredential {
            self.credentials.read(credential_id)
        }

        fn verify_credential(self: @ContractState, credential_id: felt252) -> bool {
            let credential = self.credentials.read(credential_id);
            
            // Check if revoked
            if self.revoked_credentials.read(credential_id) {
                return false;
            }
            
            // Check expiry
            if !CredentialVerificationTrait::verify_credential_expiry(credential.expires_at) {
                return false;
            }
            
            // Check if issuer is authorized
            if !self.authorized_issuers.read(credential.issuer) {
                return false;
            }
            
            // Check if subject DID is active
            let subject_did = self.dids.read(credential.subject);
            if !subject_did.is_active {
                return false;
            }
            
            true
        }

        fn create_selective_disclosure(
            ref self: ContractState,
            credential_id: felt252,
            disclosed_fields: Array<felt252>,
            proof_hash: felt252
        ) -> felt252 {
            let caller = get_caller_address();
            let credential = self.credentials.read(credential_id);
            let subject_did = self.dids.read(credential.subject);
            
            assert(caller == subject_did.controller, 'Only subject can disclose');
            assert(self.verify_credential(credential_id), 'Invalid credential');
            
            let disclosure_id = self.disclosure_counter.read() + 1;
            self.disclosure_counter.write(disclosure_id);
            
            let disclosure = SelectiveDisclosure {
                credential_id,
                disclosed_fields,
                proof_hash,
                created_at: get_block_timestamp(),
            };
            
            self.selective_disclosures.write(disclosure_id, disclosure);
            
            self.emit(SelectiveDisclosureCreated {
                disclosure_id,
                credential_id,
                creator: caller,
                timestamp: get_block_timestamp(),
            });
            
            disclosure_id
        }

        fn verify_selective_disclosure(
            self: @ContractState,
            disclosure_id: felt252,
            proof: felt252
        ) -> bool {
            let disclosure = self.selective_disclosures.read(disclosure_id);
            let credential = self.credentials.read(disclosure.credential_id);
            
            // Verify the credential is still valid
            if !self.verify_credential(disclosure.credential_id) {
                return false;
            }
            
            // Verify the selective disclosure proof
            let credential_hash = CredentialVerificationTrait::generate_credential_hash(
                credential.issuer,
                credential.subject,
                credential.credential_type,
                credential.achievement_data,
                credential.issued_at
            );
            