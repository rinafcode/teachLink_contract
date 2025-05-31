#[starknet::contract]
pub mod TeachLinkGovernance {
    use starknet::{
        get_caller_address, get_contract_address, get_block_timestamp, ContractAddress, ClassHash,
        contract_address_const, syscalls::call_contract_syscall, SyscallResultTrait,
    };
    use core::starknet::storage::{
        StoragePointerReadAccess, StoragePointerWriteAccess, Map, StoragePathEntry,
    };
    use openzeppelin::{
        access::ownable::OwnableComponent, introspection::src5::SRC5Component,
        upgrades::upgradeable::UpgradeableComponent,
        token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait},
    };
    use teachlink::interfaces::igovernance::{
        IGovernance, Proposal, Vote, Delegation, GovernanceParameters,
    };
    use teachlink::types::{status::status, vote_types::vote_types};

    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);
    component!(path: SRC5Component, storage: src5, event: SRC5Event);
    component!(path: UpgradeableComponent, storage: upgradeable, event: UpgradeableEvent);

    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    #[abi(embed_v0)]
    impl SRC5Impl = SRC5Component::SRC5Impl<ContractState>;
    impl SRC5InternalImpl = SRC5Component::InternalImpl<ContractState>;

    impl UpgradeableInternalImpl = UpgradeableComponent::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        #[substorage(v0)]
        ownable: OwnableComponent::Storage,
        #[substorage(v0)]
        src5: SRC5Component::Storage,
        #[substorage(v0)]
        upgradeable: UpgradeableComponent::Storage,
        // Core governance storage
        token: ContractAddress,
        governance_params: GovernanceParameters,
        // Proposal storage
        proposals: Map<u256, Proposal>,
        proposal_count: u256,
        proposal_snapshots: Map<u256, u64>, // proposal_id -> timestamp for voting power snapshot
        proposal_calldata: Map<(u256, u32), felt252>, // (proposal_id, index) -> calldata element
        // Voting storage
        votes: Map<(u256, ContractAddress), Vote>, // (proposal_id, voter) -> Vote
        has_voted_map: Map<(u256, ContractAddress), bool>,
        // Delegation storage
        delegates: Map<ContractAddress, ContractAddress>, // delegator -> delegate
        delegations: Map<ContractAddress, Delegation>,
        delegated_votes: Map<(ContractAddress, u64), u256>, // (delegate, timestamp) -> voting power
        // Execution storage
        proposal_eta: Map<u256, u64>, // proposal_id -> execution time
        // Initialized flag
        initialized: bool,
        // Historical balance tracking
        balance_checkpoints: Map<
            (ContractAddress, u32), BalanceCheckpoint,
        >, // (account, checkpoint_index) -> checkpoint
        checkpoint_counts: Map<ContractAddress, u32> // account -> number of checkpoints
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        #[flat]
        OwnableEvent: OwnableComponent::Event,
        #[flat]
        SRC5Event: SRC5Component::Event,
        #[flat]
        UpgradeableEvent: UpgradeableComponent::Event,
        // Governance events
        ProposalCreated: ProposalCreated,
        ProposalCanceled: ProposalCanceled,
        ProposalExecuted: ProposalExecuted,
        VoteCast: VoteCast,
        DelegateChanged: DelegateChanged,
        DelegateVotesChanged: DelegateVotesChanged,
        GovernanceParametersUpdated: GovernanceParametersUpdated,
    }

    #[derive(Drop, starknet::Event)]
    pub struct ProposalCreated {
        pub proposal_id: u256,
        pub proposer: ContractAddress,
        pub target: ContractAddress,
        pub title: ByteArray,
        pub description: ByteArray,
        pub start_time: u64,
        pub end_time: u64,
    }

    #[derive(Drop, starknet::Event)]
    pub struct ProposalCanceled {
        pub proposal_id: u256,
    }

    #[derive(Drop, starknet::Event)]
    pub struct ProposalExecuted {
        pub proposal_id: u256,
    }

    #[derive(Drop, starknet::Event)]
    pub struct VoteCast {
        pub voter: ContractAddress,
        pub proposal_id: u256,
        pub support: u8,
        pub weight: u256,
        pub reason: ByteArray,
    }

    #[derive(Drop, starknet::Event)]
    pub struct DelegateChanged {
        pub delegator: ContractAddress,
        pub from_delegate: ContractAddress,
        pub to_delegate: ContractAddress,
    }

    #[derive(Drop, starknet::Event)]
    pub struct DelegateVotesChanged {
        pub delegate: ContractAddress,
        pub previous_balance: u256,
        pub new_balance: u256,
    }

    #[derive(Drop, starknet::Event)]
    pub struct GovernanceParametersUpdated {
        pub voting_delay: u64,
        pub voting_period: u64,
        pub proposal_threshold: u256,
        pub quorum_threshold: u256,
        pub execution_delay: u64,
    }

    #[derive(Drop, Serde, starknet::Store)]
    pub struct BalanceCheckpoint {
        pub timestamp: u64,
        pub balance: u256,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        let caller = get_caller_address();
        self.ownable.initializer(caller);
        self.initialized.write(false);
    }

    #[abi(embed_v0)]
    impl TeachLinkGovernanceImpl of IGovernance<ContractState> {
        fn initialize(
            ref self: ContractState,
            token_address: ContractAddress,
            initial_params: GovernanceParameters,
        ) {
            self.ownable.assert_only_owner();
            assert!(!self.initialized.read(), "Already initialized");

            self.token.write(token_address);
            self.governance_params.write(initial_params);
            self.proposal_count.write(0);
            self.initialized.write(true);
        }

        fn create_proposal(
            ref self: ContractState,
            title: ByteArray,
            description: ByteArray,
            target: ContractAddress,
            calldata: Span<felt252>,
            value: u256,
        ) -> u256 {
            self._assert_initialized();
            let caller = get_caller_address();
            let current_time = get_block_timestamp();
            let params = self.governance_params.read();

            // Check proposal threshold
            let voting_power = self._get_current_voting_power(caller);
            assert!(voting_power >= params.proposal_threshold, "Insufficient voting power");

            // Create new proposal
            let proposal_id = self.proposal_count.read() + 1;
            self.proposal_count.write(proposal_id);

            let start_time = current_time + params.voting_delay;
            let end_time = start_time + params.voting_period;

            // Store calldata separately
            let calldata_len = calldata.len();
            let mut i = 0;
            while i < calldata_len {
                self.proposal_calldata.entry((proposal_id, i)).write(*calldata.at(i));
                i += 1;
            };

            let proposal = Proposal {
                id: proposal_id,
                title: title.clone(),
                description: description.clone(),
                proposer: caller,
                target,
                calldata_len,
                value,
                votes_for: 0,
                votes_against: 0,
                votes_abstain: 0,
                start_time,
                end_time,
                executed: false,
                canceled: false,
            };

            self.proposals.entry(proposal_id).write(proposal);
            self.proposal_snapshots.entry(proposal_id).write(start_time);

            self
                .emit(
                    ProposalCreated {
                        proposal_id,
                        proposer: caller,
                        target,
                        title,
                        description,
                        start_time,
                        end_time,
                    },
                );

            proposal_id
        }

        fn cancel_proposal(ref self: ContractState, proposal_id: u256) {
            self._assert_initialized();
            let caller = get_caller_address();
            let mut proposal = self.proposals.entry(proposal_id).read();

            // Only proposer or owner can cancel
            assert!(
                caller == proposal.proposer || caller == self.ownable.owner(),
                "Unauthorized to cancel",
            );

            let state = self._get_proposal_state_internal(proposal_id);
            assert!(state == status::PENDING || state == status::ACTIVE, "Cannot cancel proposal");

            proposal.canceled = true;
            self.proposals.entry(proposal_id).write(proposal);

            self.emit(ProposalCanceled { proposal_id });
        }

        fn execute_proposal(ref self: ContractState, proposal_id: u256) {
            self._assert_initialized();
            let proposal = self.proposals.entry(proposal_id).read();
            let state = self._get_proposal_state_internal(proposal_id);

            assert!(state == status::SUCCEEDED, "Not in succeeded state");

            let current_time = get_block_timestamp();
            let params = self.governance_params.read();
            let eta = self.proposal_eta.entry(proposal_id).read();

            if eta == 0 {
                // Queue the proposal for execution
                let execution_time = current_time + params.execution_delay;
                self.proposal_eta.entry(proposal_id).write(execution_time);
                return;
            }

            assert!(current_time >= eta, "Not ready for execution");

            // Execute the proposal
            if proposal.target != contract_address_const::<0>() {
                // Reconstruct calldata
                let mut calldata_array = array![];
                let mut i = 0;
                while i < proposal.calldata_len {
                    calldata_array.append(self.proposal_calldata.entry((proposal_id, i)).read());
                    i += 1;
                };

                call_contract_syscall(proposal.target, 0, calldata_array.span()).unwrap_syscall();
            }

            let mut updated_proposal = proposal;
            updated_proposal.executed = true;
            self.proposals.entry(proposal_id).write(updated_proposal);

            self.emit(ProposalExecuted { proposal_id });
        }

        fn cast_vote(ref self: ContractState, proposal_id: u256, support: u8, reason: ByteArray) {
            self._assert_initialized();
            let voter = get_caller_address();
            self._cast_vote_internal(voter, proposal_id, support, reason);
        }


        fn delegate(ref self: ContractState, delegate: ContractAddress) {
            self._assert_initialized();
            let delegator = get_caller_address();
            self._delegate_internal(delegator, delegate);
        }

        fn undelegate(ref self: ContractState) {
            self._assert_initialized();
            let delegator = get_caller_address();
            self._delegate_internal(delegator, delegator);
        }


        fn get_proposal(self: @ContractState, proposal_id: u256) -> Proposal {
            self.proposals.entry(proposal_id).read()
        }

        fn get_proposal_state(self: @ContractState, proposal_id: u256) -> u8 {
            self._get_proposal_state_internal(proposal_id)
        }

        fn get_vote(self: @ContractState, proposal_id: u256, voter: ContractAddress) -> Vote {
            self.votes.entry((proposal_id, voter)).read()
        }

        fn get_voting_power(
            self: @ContractState, account: ContractAddress, timestamp: u64,
        ) -> u256 {
            // Get delegated voting power at specific timestamp
            let delegate = self.delegates.entry(account).read();
            if delegate != contract_address_const::<0>() {
                self.delegated_votes.entry((delegate, timestamp)).read()
            } else {
                self._get_token_balance_at_timestamp(account, timestamp)
            }
        }

        fn get_delegate(self: @ContractState, account: ContractAddress) -> ContractAddress {
            let delegate = self.delegates.entry(account).read();
            if delegate != contract_address_const::<0>() {
                delegate
            } else {
                account
            }
        }

        fn get_delegation(self: @ContractState, delegator: ContractAddress) -> Delegation {
            self.delegations.entry(delegator).read()
        }

        fn get_governance_parameters(self: @ContractState) -> GovernanceParameters {
            self.governance_params.read()
        }

        fn get_proposal_count(self: @ContractState) -> u256 {
            self.proposal_count.read()
        }

        fn has_voted(self: @ContractState, proposal_id: u256, voter: ContractAddress) -> bool {
            self.has_voted_map.entry((proposal_id, voter)).read()
        }

        fn update_governance_parameters(
            ref self: ContractState,
            voting_delay: u64,
            voting_period: u64,
            proposal_threshold: u256,
            quorum_threshold: u256,
            execution_delay: u64,
        ) {
            // called through governance execution
            let caller = get_caller_address();
            assert!(caller == get_contract_address(), "Only governance can update");

            let new_params = GovernanceParameters {
                voting_delay, voting_period, proposal_threshold, quorum_threshold, execution_delay,
            };

            self.governance_params.write(new_params);

            self
                .emit(
                    GovernanceParametersUpdated {
                        voting_delay,
                        voting_period,
                        proposal_threshold,
                        quorum_threshold,
                        execution_delay,
                    },
                );
        }

        fn upgrade(ref self: ContractState, new_class_hash: ClassHash) {
            // called through governance execution
            let caller = get_caller_address();
            assert!(caller == get_contract_address(), "Only governance can upgrade");

            // Replace the class hash upgrading the contract
            self.upgradeable.upgrade(new_class_hash);
        }
    }

    #[generate_trait]
    impl InternalFunctions of InternalFunctionsTrait {
        fn _assert_initialized(self: @ContractState) {
            assert!(self.initialized.read(), "Contract not initialized");
        }

        fn _cast_vote_internal(
            ref self: ContractState,
            voter: ContractAddress,
            proposal_id: u256,
            support: u8,
            reason: ByteArray,
        ) {
            assert!(support <= 2, "Invalid support value");
            assert!(!self.has_voted_map.entry((proposal_id, voter)).read(), "Already voted");

            let proposal = self.proposals.entry(proposal_id).read();
            let state = self._get_proposal_state_internal(proposal_id);
            assert!(state == status::ACTIVE, "Proposal not active");

            let snapshot_time = self.proposal_snapshots.entry(proposal_id).read();
            let voting_power = self.get_voting_power(voter, snapshot_time);
            assert!(voting_power > 0, "No voting power");

            // Record the vote
            let vote = Vote {
                voter, proposal_id, support, weight: voting_power, reason: reason.clone(),
            };

            self.votes.entry((proposal_id, voter)).write(vote);
            self.has_voted_map.entry((proposal_id, voter)).write(true);

            // Update proposal vote counts
            let mut updated_proposal = proposal;
            if support == vote_types::VOTE_FOR {
                updated_proposal.votes_for += voting_power;
            } else if support == vote_types::VOTE_AGAINST {
                updated_proposal.votes_against += voting_power;
            } else {
                updated_proposal.votes_abstain += voting_power;
            }

            self.proposals.entry(proposal_id).write(updated_proposal);

            self.emit(VoteCast { voter, proposal_id, support, weight: voting_power, reason });
        }

        fn _delegate_internal(
            ref self: ContractState, delegator: ContractAddress, delegate: ContractAddress,
        ) {
            let current_delegate = self.delegates.entry(delegator).read();
            let current_time = get_block_timestamp();

            if current_delegate != delegate {
                self.delegates.entry(delegator).write(delegate);

                let delegation = Delegation { delegator, delegate, timestamp: current_time };
                self.delegations.entry(delegator).write(delegation);

                // Update delegated voting power
                let voting_power = self._get_current_voting_power(delegator);

                // Remove from old delegate
                if current_delegate != contract_address_const::<0>()
                    && current_delegate != delegator {
                    let old_power = self
                        .delegated_votes
                        .entry((current_delegate, current_time))
                        .read();
                    self
                        .delegated_votes
                        .entry((current_delegate, current_time))
                        .write(old_power - voting_power);
                }

                // Add to new delegate
                if delegate != delegator {
                    let new_power = self.delegated_votes.entry((delegate, current_time)).read();
                    self
                        .delegated_votes
                        .entry((delegate, current_time))
                        .write(new_power + voting_power);
                }

                self
                    .emit(
                        DelegateChanged {
                            delegator, from_delegate: current_delegate, to_delegate: delegate,
                        },
                    );
            }
        }

        fn _get_proposal_state_internal(self: @ContractState, proposal_id: u256) -> u8 {
            let proposal = self.proposals.entry(proposal_id).read();

            if proposal.canceled {
                return status::CANCELED;
            }

            if proposal.executed {
                return status::EXECUTED;
            }

            let current_time = get_block_timestamp();

            if current_time < proposal.start_time {
                return status::PENDING;
            }

            if current_time <= proposal.end_time {
                return status::ACTIVE;
            }

            // Voting has ended, check results
            let params = self.governance_params.read();
            let total_votes = proposal.votes_for + proposal.votes_against + proposal.votes_abstain;

            if total_votes < params.quorum_threshold {
                return status::DEFEATED;
            }

            if proposal.votes_for > proposal.votes_against {
                let eta = self.proposal_eta.entry(proposal_id).read();
                if eta == 0 {
                    return status::SUCCEEDED;
                } else if current_time >= eta + params.execution_delay {
                    return status::EXPIRED;
                } else {
                    return status::QUEUED;
                }
            } else {
                return status::DEFEATED;
            }
        }

        fn _get_current_voting_power(self: @ContractState, account: ContractAddress) -> u256 {
            let token = IERC20Dispatcher { contract_address: self.token.read() };
            token.balance_of(account)
        }

        fn _get_token_balance_at_timestamp(
            self: @ContractState, account: ContractAddress, timestamp: u64,
        ) -> u256 {
            let checkpoint_count = self.checkpoint_counts.entry(account).read();

            if checkpoint_count == 0 {
                // No checkpoints, return current balance
                return self._get_current_voting_power(account);
            }

            // Binary search for the appropriate checkpoint
            let mut low: u32 = 0;
            let mut high: u32 = checkpoint_count - 1;

            while low < high {
                let mid = (low + high + 1) / 2;
                let checkpoint = self.balance_checkpoints.entry((account, mid)).read();

                if checkpoint.timestamp <= timestamp {
                    low = mid;
                } else {
                    high = mid - 1;
                }
            };

            let checkpoint = self.balance_checkpoints.entry((account, low)).read();

            // If the checkpoint timestamp is after the requested timestamp,
            // return 0 (no balance at that time)
            if checkpoint.timestamp > timestamp {
                0
            } else {
                checkpoint.balance
            }
        }


        fn _update_balance_checkpoint(
            ref self: ContractState, account: ContractAddress, new_balance: u256,
        ) {
            let current_time = get_block_timestamp();
            let checkpoint_count = self.checkpoint_counts.entry(account).read();

            if checkpoint_count > 0 {
                // Check if we can update the latest checkpoint (same timestamp)
                let latest_checkpoint = self
                    .balance_checkpoints
                    .entry((account, checkpoint_count - 1))
                    .read();

                if latest_checkpoint.timestamp == current_time {
                    // Update existing checkpoint for same timestamp
                    let updated_checkpoint = BalanceCheckpoint {
                        timestamp: current_time, balance: new_balance,
                    };
                    self
                        .balance_checkpoints
                        .entry((account, checkpoint_count - 1))
                        .write(updated_checkpoint);
                    return;
                }
            }

            // Create new checkpoint
            let new_checkpoint = BalanceCheckpoint {
                timestamp: current_time, balance: new_balance,
            };

            self.balance_checkpoints.entry((account, checkpoint_count)).write(new_checkpoint);
            self.checkpoint_counts.entry(account).write(checkpoint_count + 1);
        }
    }
}
