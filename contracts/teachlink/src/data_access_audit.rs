use soroban_sdk::{contract, contractimpl, Address, Env, Map, Symbol, Vec, U256};

#[contract]
pub struct DataAccessAuditContract;

#[derive(Clone, Debug)]
pub struct AccessLog {
    pub id: U256,
    pub student_id: Address,
    pub accessor: Address,
    pub timestamp: u64,
    pub access_type: Symbol,
    pub purpose: Symbol,
}

impl AccessLog {
    pub fn new(
        id: U256,
        student_id: Address,
        accessor: Address,
        timestamp: u64,
        access_type: Symbol,
        purpose: Symbol,
    ) -> Self {
        Self {
            id,
            student_id,
            accessor,
            timestamp,
            access_type,
            purpose,
        }
    }
}

pub struct DataAccessAuditData {
    pub access_logs: Map<U256, AccessLog>,
    pub student_logs: Map<Address, Vec<U256>>,
    pub accessor_logs: Map<Address, Vec<U256>>,
    pub next_log_id: U256,
}

impl DataAccessAuditData {
    const ACCESS_LOGS: Symbol = Symbol::new(&Env::default(), "ACCESS_LOGS");
    const STUDENT_LOGS: Symbol = Symbol::new(&Env::default(), "STUDENT_LOGS");
    const ACCESSOR_LOGS: Symbol = Symbol::new(&Env::default(), "ACCESSOR_LOGS");
    const NEXT_LOG_ID: Symbol = Symbol::new(&Env::default(), "NEXT_LOG_ID");

    pub fn new(env: &Env) -> Self {
        Self {
            access_logs: Map::new(env),
            student_logs: Map::new(env),
            accessor_logs: Map::new(env),
            next_log_id: U256::from_u32(1),
        }
    }

    fn get_access_logs(env: &Env) -> Map<U256, AccessLog> {
        env.storage()
            .instance()
            .get(&Self::ACCESS_LOGS)
            .unwrap_or_else(|| Map::new(env))
    }

    fn set_access_logs(env: &Env, access_logs: &Map<U256, AccessLog>) {
        env.storage().instance().set(&Self::ACCESS_LOGS, access_logs);
    }

    fn get_student_logs(env: &Env) -> Map<Address, Vec<U256>> {
        env.storage()
            .instance()
            .get(&Self::STUDENT_LOGS)
            .unwrap_or_else(|| Map::new(env))
    }

    fn set_student_logs(env: &Env, student_logs: &Map<Address, Vec<U256>>) {
        env.storage().instance().set(&Self::STUDENT_LOGS, student_logs);
    }

    fn get_accessor_logs(env: &Env) -> Map<Address, Vec<U256>> {
        env.storage()
            .instance()
            .get(&Self::ACCESSOR_LOGS)
            .unwrap_or_else(|| Map::new(env))
    }

    fn set_accessor_logs(env: &Env, accessor_logs: &Map<Address, Vec<U256>>) {
        env.storage().instance().set(&Self::ACCESSOR_LOGS, accessor_logs);
    }

    fn get_next_log_id(env: &Env) -> U256 {
        env.storage()
            .instance()
            .get(&Self::NEXT_LOG_ID)
            .unwrap_or_else(|| U256::from_u32(1))
    }

    fn set_next_log_id(env: &Env, next_log_id: U256) {
        env.storage().instance().set(&Self::NEXT_LOG_ID, &next_log_id);
    }

    pub fn log_access(
        env: &Env,
        student_id: Address,
        accessor: Address,
        access_type: Symbol,
        purpose: Symbol,
    ) -> U256 {
        let log_id = Self::get_next_log_id(env);
        let timestamp = env.ledger().timestamp();

        let access_log = AccessLog::new(
            log_id,
            student_id.clone(),
            accessor.clone(),
            timestamp,
            access_type,
            purpose,
        );

        let mut access_logs = Self::get_access_logs(env);
        access_logs.set(log_id, access_log.clone());
        Self::set_access_logs(env, &access_logs);

        let mut student_logs = Self::get_student_logs(env);
        let mut student_log_list = student_logs.get(student_id.clone()).unwrap_or_else(|| Vec::new(env));
        student_log_list.push_back(log_id);
        student_logs.set(student_id, student_log_list);
        Self::set_student_logs(env, &student_logs);

        let mut accessor_logs = Self::get_accessor_logs(env);
        let mut accessor_log_list = accessor_logs.get(accessor.clone()).unwrap_or_else(|| Vec::new(env));
        accessor_log_list.push_back(log_id);
        accessor_logs.set(accessor, accessor_log_list);
        Self::set_accessor_logs(env, &accessor_logs);

        Self::set_next_log_id(env, log_id + U256::from_u32(1));

        log_id
    }

    pub fn get_access_logs(env: &Env, student_id: Address) -> Vec<AccessLog> {
        let student_logs = Self::get_student_logs(env);
        let access_logs = Self::get_access_logs(env);
        
        if let Some(log_ids) = student_logs.get(student_id) {
            log_ids.iter()
                .map(|log_id| access_logs.get(log_id).unwrap())
                .collect()
        } else {
            Vec::new(env)
        }
    }

    pub fn get_accessor_logs(env: &Env, accessor: Address) -> Vec<AccessLog> {
        let accessor_logs = Self::get_accessor_logs(env);
        let access_logs = Self::get_access_logs(env);
        
        if let Some(log_ids) = accessor_logs.get(accessor) {
            log_ids.iter()
                .map(|log_id| access_logs.get(log_id).unwrap())
                .collect()
        } else {
            Vec::new(env)
        }
    }

    pub fn get_access_log(env: &Env, log_id: U256) -> AccessLog {
        Self::get_access_logs(env)
            .get(log_id)
            .unwrap_or_else(|| panic_with_error!(env, "Access log not found"))
    }

    pub fn get_access_logs_by_time_range(
        env: &Env,
        student_id: Address,
        start_time: u64,
        end_time: u64,
    ) -> Vec<AccessLog> {
        let all_logs = Self::get_access_logs(env, student_id);
        
        all_logs.iter()
            .filter(|log| log.timestamp >= start_time && log.timestamp <= end_time)
            .collect()
    }

    pub fn get_access_logs_by_type(
        env: &Env,
        student_id: Address,
        access_type: Symbol,
    ) -> Vec<AccessLog> {
        let all_logs = Self::get_access_logs(env, student_id);
        
        all_logs.iter()
            .filter(|log| log.access_type == access_type)
            .collect()
    }

    pub fn get_total_access_count(env: &Env, student_id: Address) -> u32 {
        let student_logs = Self::get_student_logs(env);
        
        if let Some(log_ids) = student_logs.get(student_id) {
            log_ids.len() as u32
        } else {
            0
        }
    }

    pub fn get_access_summary(env: &Env, student_id: Address) -> Map<Symbol, u32> {
        let all_logs = Self::get_access_logs(env, student_id);
        let mut summary = Map::new(env);
        
        for log in all_logs.iter() {
            let count = summary.get(log.access_type.clone()).unwrap_or(0);
            summary.set(log.access_type, count + 1);
        }
        
        summary
    }
}

#[contractimpl]
impl DataAccessAuditContract {
    pub fn log_access(
        env: Env,
        student_id: Address,
        accessor: Address,
        access_type: Symbol,
        purpose: Symbol,
    ) -> U256 {
        DataAccessAuditData::log_access(&env, student_id, accessor, access_type, purpose)
    }

    pub fn get_access_logs(env: Env, student_id: Address) -> Vec<AccessLog> {
        DataAccessAuditData::get_access_logs(&env, student_id)
    }

    pub fn get_accessor_logs(env: Env, accessor: Address) -> Vec<AccessLog> {
        DataAccessAuditData::get_accessor_logs(&env, accessor)
    }

    pub fn get_access_log(env: Env, log_id: U256) -> AccessLog {
        DataAccessAuditData::get_access_log(&env, log_id)
    }

    pub fn get_access_logs_by_time_range(
        env: Env,
        student_id: Address,
        start_time: u64,
        end_time: u64,
    ) -> Vec<AccessLog> {
        DataAccessAuditData::get_access_logs_by_time_range(&env, student_id, start_time, end_time)
    }

    pub fn get_access_logs_by_type(
        env: Env,
        student_id: Address,
        access_type: Symbol,
    ) -> Vec<AccessLog> {
        DataAccessAuditData::get_access_logs_by_type(&env, student_id, access_type)
    }

    pub fn get_total_access_count(env: Env, student_id: Address) -> u32 {
        DataAccessAuditData::get_total_access_count(&env, student_id)
    }

    pub fn get_access_summary(env: Env, student_id: Address) -> Map<Symbol, u32> {
        DataAccessAuditData::get_access_summary(&env, student_id)
    }

    pub fn verify_log_integrity(env: Env, log_id: U256) -> bool {
        let access_logs = DataAccessAuditData::get_access_logs(&env);
        access_logs.contains_key(log_id)
    }
}
