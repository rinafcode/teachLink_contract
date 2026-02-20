//! Governance Compliance and Regulatory Reporting
//!
//! Tracks audit logs and generates reports for regulatory standards.
//! Ensures all governance actions are traceable and compliant with
//! decentralization milestones.

use soroban_sdk::{contracttype, Address, Bytes, Env, symbol_short, Symbol, Vec};

const REPORTS: Symbol = symbol_short!("reports");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplianceReport {
    pub timestamp: u64,
    pub total_proposals: u64,
    pub total_voters: u32,
    pub decentralization_ratio: u32, // Ratio of non-admin voting power (basis points)
    pub audit_hash: Bytes,
}

pub struct Compliance;

impl Compliance {
    pub fn generate_report(
        env: &Env,
        admin: Address,
        total_p: u64,
        voters: u32,
        ratio: u32,
        hash: Bytes
    ) -> ComplianceReport {
        admin.require_auth();
        
        let report = ComplianceReport {
            timestamp: env.ledger().timestamp(),
            total_proposals: total_p,
            total_voters: voters,
            decentralization_ratio: ratio,
            audit_hash: hash,
        };

        let mut all_reports: Vec<ComplianceReport> = env.storage().instance().get(&REPORTS).unwrap_or(Vec::new(env));
        all_reports.push_back(report.clone());
        env.storage().instance().set(&REPORTS, &all_reports);
        
        report
    }

    pub fn get_latest_reports(env: &Env) -> Vec<ComplianceReport> {
        env.storage().instance().get(&REPORTS).unwrap_or(Vec::new(env))
    }
}
