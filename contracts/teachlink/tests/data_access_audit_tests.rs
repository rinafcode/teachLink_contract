use teachlink_contract::{AccessLog, DataAccessAuditContract};
use soroban_sdk::{symbol_short, Address, Env, Map, U256};

#[test]
fn test_access_logging() {
    let env = Env::default();
    let student_id = Address::generate(&env);
    let accessor = Address::generate(&env);
    let access_type = symbol_short!("Read");
    let purpose = symbol_short!("Treatment");
    
    let log_id = DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor.clone(),
        access_type,
        purpose,
    );
    
    let log = DataAccessAuditContract::get_access_log(env.clone(), log_id);
    assert_eq!(log.student_id, student_id);
    assert_eq!(log.accessor, accessor);
    assert_eq!(log.access_type, symbol_short!("Read"));
    assert_eq!(log.purpose, symbol_short!("Treatment"));
}

#[test]
fn test_student_access_logs() {
    let env = Env::default();
    let student_id = Address::generate(&env);
    let accessor1 = Address::generate(&env);
    let accessor2 = Address::generate(&env);
    
    let log_id1 = DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor1.clone(),
        symbol_short!("Read"),
        symbol_short!("Treatment"),
    );
    
    let log_id2 = DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor2.clone(),
        symbol_short!("Write"),
        symbol_short!("Update"),
    );
    
    let student_logs = DataAccessAuditContract::get_access_logs(env.clone(), student_id.clone());
    assert_eq!(student_logs.len(), 2);
    
    let log_ids: Vec<U256> = student_logs.iter().map(|log| log.id).collect();
    assert!(log_ids.contains(log_id1));
    assert!(log_ids.contains(log_id2));
}

#[test]
fn test_accessor_logs() {
    let env = Env::default();
    let student1 = Address::generate(&env);
    let student2 = Address::generate(&env);
    let accessor = Address::generate(&env);
    
    let log_id1 = DataAccessAuditContract::log_access(
        env.clone(),
        student1.clone(),
        accessor.clone(),
        symbol_short!("Read"),
        symbol_short!("Treatment"),
    );
    
    let log_id2 = DataAccessAuditContract::log_access(
        env.clone(),
        student2.clone(),
        accessor.clone(),
        symbol_short!("Write"),
        symbol_short!("Update"),
    );
    
    let accessor_logs = DataAccessAuditContract::get_accessor_logs(env.clone(), accessor.clone());
    assert_eq!(accessor_logs.len(), 2);
    
    let log_ids: Vec<U256> = accessor_logs.iter().map(|log| log.id).collect();
    assert!(log_ids.contains(log_id1));
    assert!(log_ids.contains(log_id2));
}

#[test]
fn test_access_logs_by_type() {
    let env = Env::default();
    let student_id = Address::generate(&env);
    let accessor = Address::generate(&env);
    
    DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor.clone(),
        symbol_short!("Read"),
        symbol_short!("Treatment"),
    );
    
    DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor.clone(),
        symbol_short!("Read"),
        symbol_short!("Diagnosis"),
    );
    
    DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor.clone(),
        symbol_short!("Write"),
        symbol_short!("Update"),
    );
    
    let read_logs = DataAccessAuditContract::get_access_logs_by_type(
        env.clone(),
        student_id.clone(),
        symbol_short!("Read"),
    );
    assert_eq!(read_logs.len(), 2);
    
    let write_logs = DataAccessAuditContract::get_access_logs_by_type(
        env.clone(),
        student_id.clone(),
        symbol_short!("Write"),
    );
    assert_eq!(write_logs.len(), 1);
}

#[test]
fn test_total_access_count() {
    let env = Env::default();
    let student_id = Address::generate(&env);
    let accessor = Address::generate(&env);
    
    DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor.clone(),
        symbol_short!("Read"),
        symbol_short!("Treatment"),
    );
    
    DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor.clone(),
        symbol_short!("Write"),
        symbol_short!("Update"),
    );
    
    let count = DataAccessAuditContract::get_total_access_count(env.clone(), student_id.clone());
    assert_eq!(count, 2);
}

#[test]
fn test_access_summary() {
    let env = Env::default();
    let student_id = Address::generate(&env);
    let accessor = Address::generate(&env);
    
    DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor.clone(),
        symbol_short!("Read"),
        symbol_short!("Treatment"),
    );
    
    DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor.clone(),
        symbol_short!("Read"),
        symbol_short!("Diagnosis"),
    );
    
    DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor.clone(),
        symbol_short!("Write"),
        symbol_short!("Update"),
    );
    
    let summary = DataAccessAuditContract::get_access_summary(env.clone(), student_id.clone());
    assert_eq!(summary.get(symbol_short!("Read")).unwrap(), 2);
    assert_eq!(summary.get(symbol_short!("Write")).unwrap(), 1);
}

#[test]
fn test_log_integrity_verification() {
    let env = Env::default();
    let student_id = Address::generate(&env);
    let accessor = Address::generate(&env);
    
    let log_id = DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor.clone(),
        symbol_short!("Read"),
        symbol_short!("Treatment"),
    );
    
    assert!(DataAccessAuditContract::verify_log_integrity(env.clone(), log_id));
    assert!(!DataAccessAuditContract::verify_log_integrity(env.clone(), U256::from_u32(999)));
}

#[test]
fn test_access_logs_by_time_range() {
    let env = Env::default();
    let student_id = Address::generate(&env);
    let accessor = Address::generate(&env);
    
    // Set a specific timestamp for testing
    env.ledger().set_timestamp(1000);
    
    let log_id1 = DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor.clone(),
        symbol_short!("Read"),
        symbol_short!("Treatment"),
    );
    
    env.ledger().set_timestamp(2000);
    
    let log_id2 = DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor.clone(),
        symbol_short!("Write"),
        symbol_short!("Update"),
    );
    
    env.ledger().set_timestamp(3000);
    
    let log_id3 = DataAccessAuditContract::log_access(
        env.clone(),
        student_id.clone(),
        accessor.clone(),
        symbol_short!("Read"),
        symbol_short!("Diagnosis"),
    );
    
    // Get logs in time range 1500-2500 (should include log_id2 only)
    let logs_in_range = DataAccessAuditContract::get_access_logs_by_time_range(
        env.clone(),
        student_id.clone(),
        1500,
        2500,
    );
    assert_eq!(logs_in_range.len(), 1);
    assert_eq!(logs_in_range.get(0).unwrap().id, log_id2);
    
    // Get logs in time range 500-3500 (should include all logs)
    let all_logs = DataAccessAuditContract::get_access_logs_by_time_range(
        env.clone(),
        student_id.clone(),
        500,
        3500,
    );
    assert_eq!(all_logs.len(), 3);
}
