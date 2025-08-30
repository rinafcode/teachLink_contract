use starknet::ContractAddress;
use core::pedersen::pedersen;
use core::poseidon::poseidon_hash_span;

#[derive(Drop, Serde, starknet::Store)]
struct CompletionProof {
    student: ContractAddress,
    course_id: u256,
    completion_percentage: u8,
    assignments_completed: u32,
    quiz_scores: Array<u8>,
    participation_score: u32,
    timestamp: u64,
    instructor_signature: felt252,
}

#[derive(Drop, Serde, starknet::Store)]
struct VerificationResult {
    is_valid: bool,
    completion_percentage: u8,
    meets_requirements: bool,
    verification_timestamp: u64,
    verification_hash: felt252,
}

mod CertificateVerification {
    use super::{CompletionProof, VerificationResult, CourseRequirements};
    use starknet::{ContractAddress, get_block_timestamp};
    use core::pedersen::pedersen;
    use core::poseidon::poseidon_hash_span;
    use core::array::ArrayTrait;

    fn verify_completion_proof(
        proof: CompletionProof,
        requirements: CourseRequirements
    ) -> VerificationResult {
        let mut is_valid = true;
        let mut meets_requirements = true;

        // Verify completion percentage
        if proof.completion_percentage < requirements.min_completion_percentage {
            meets_requirements = false;
        }

        // Verify assignments completed
        if proof.assignments_completed < requirements.required_assignments {
            meets_requirements = false;
        }

        // Verify quiz scores
        let avg_quiz_score = calculate_average_quiz_score(proof.quiz_scores.span());
        if avg_quiz_score < requirements.min_quiz_score {
            meets_requirements = false;
        }
