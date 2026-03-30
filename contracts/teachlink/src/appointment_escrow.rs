use soroban_sdk::{contract, contractimpl, Address, Env, Map, Symbol, Vec, panic_with_error, U256};

#[contract]
pub struct AppointmentEscrowContract;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppointmentStatus {
    Booked,
    Confirmed,
    Completed,
    Canceled,
    Refunded,
}

impl AppointmentStatus {
    pub fn to_symbol(&self) -> Symbol {
        match self {
            AppointmentStatus::Booked => Symbol::new(&Env::default(), "Booked"),
            AppointmentStatus::Confirmed => Symbol::new(&Env::default(), "Confirmed"),
            AppointmentStatus::Completed => Symbol::new(&Env::default(), "Completed"),
            AppointmentStatus::Canceled => Symbol::new(&Env::default(), "Canceled"),
            AppointmentStatus::Refunded => Symbol::new(&Env::default(), "Refunded"),
        }
    }

    pub fn from_symbol(env: &Env, symbol: &Symbol) -> AppointmentStatus {
        if *symbol == Symbol::new(env, "Booked") {
            AppointmentStatus::Booked
        } else if *symbol == Symbol::new(env, "Confirmed") {
            AppointmentStatus::Confirmed
        } else if *symbol == Symbol::new(env, "Completed") {
            AppointmentStatus::Completed
        } else if *symbol == Symbol::new(env, "Canceled") {
            AppointmentStatus::Canceled
        } else if *symbol == Symbol::new(env, "Refunded") {
            AppointmentStatus::Refunded
        } else {
            panic_with_error!(env, "Invalid appointment status");
        }
    }
}

#[derive(Clone, Debug)]
pub struct Appointment {
    pub id: U256,
    pub student: Address,
    pub provider: Address,
    pub amount: U256,
    pub status: AppointmentStatus,
    pub timestamp: u64,
    pub token_address: Address,
}

pub struct AppointmentEscrowData {
    pub appointments: Map<U256, Appointment>,
    pub student_appointments: Map<Address, Vec<U256>>,
    pub provider_appointments: Map<Address, Vec<U256>>,
    pub next_appointment_id: U256,
}

impl AppointmentEscrowData {
    const APPOINTMENTS: Symbol = Symbol::new(&Env::default(), "APPOINTMENTS");
    const STUDENT_APPOINTMENTS: Symbol = Symbol::new(&Env::default(), "STUDENT_APPOINTMENTS");
    const PROVIDER_APPOINTMENTS: Symbol = Symbol::new(&Env::default(), "PROVIDER_APPOINTMENTS");
    const NEXT_ID: Symbol = Symbol::new(&Env::default(), "NEXT_ID");

    pub fn new(env: &Env) -> Self {
        Self {
            appointments: Map::new(env),
            student_appointments: Map::new(env),
            provider_appointments: Map::new(env),
            next_appointment_id: U256::from_u32(1),
        }
    }

    fn get_appointments(env: &Env) -> Map<U256, Appointment> {
        env.storage()
            .instance()
            .get(&Self::APPOINTMENTS)
            .unwrap_or_else(|| Map::new(env))
    }

    fn set_appointments(env: &Env, appointments: &Map<U256, Appointment>) {
        env.storage().instance().set(&Self::APPOINTMENTS, appointments);
    }

    fn get_student_appointments(env: &Env) -> Map<Address, Vec<U256>> {
        env.storage()
            .instance()
            .get(&Self::STUDENT_APPOINTMENTS)
            .unwrap_or_else(|| Map::new(env))
    }

    fn set_student_appointments(env: &Env, student_appointments: &Map<Address, Vec<U256>>) {
        env.storage().instance().set(&Self::STUDENT_APPOINTMENTS, student_appointments);
    }

    fn get_provider_appointments(env: &Env) -> Map<Address, Vec<U256>> {
        env.storage()
            .instance()
            .get(&Self::PROVIDER_APPOINTMENTS)
            .unwrap_or_else(|| Map::new(env))
    }

    fn set_provider_appointments(env: &Env, provider_appointments: &Map<Address, Vec<U256>>) {
        env.storage().instance().set(&Self::PROVIDER_APPOINTMENTS, provider_appointments);
    }

    fn get_next_id(env: &Env) -> U256 {
        env.storage()
            .instance()
            .get(&Self::NEXT_ID)
            .unwrap_or_else(|| U256::from_u32(1))
    }

    fn set_next_id(env: &Env, next_id: U256) {
        env.storage().instance().set(&Self::NEXT_ID, &next_id);
    }

    pub fn book_appointment(
        env: &Env,
        student: Address,
        provider: Address,
        amount: U256,
        token_address: Address,
    ) -> U256 {
        let appointment_id = Self::get_next_id(env);
        let timestamp = env.ledger().timestamp();

        let appointment = Appointment {
            id: appointment_id,
            student: student.clone(),
            provider: provider.clone(),
            amount,
            status: AppointmentStatus::Booked,
            timestamp,
            token_address: token_address.clone(),
        };

        let mut appointments = Self::get_appointments(env);
        appointments.set(appointment_id, appointment.clone());
        Self::set_appointments(env, &appointments);

        let mut student_appointments = Self::get_student_appointments(env);
        let mut student_list = student_appointments.get(student.clone()).unwrap_or_else(|| Vec::new(env));
        student_list.push_back(appointment_id);
        student_appointments.set(student, student_list);
        Self::set_student_appointments(env, &student_appointments);

        let mut provider_appointments = Self::get_provider_appointments(env);
        let mut provider_list = provider_appointments.get(provider.clone()).unwrap_or_else(|| Vec::new(env));
        provider_list.push_back(appointment_id);
        provider_appointments.set(provider, provider_list);
        Self::set_provider_appointments(env, &provider_appointments);

        Self::set_next_id(env, appointment_id + U256::from_u32(1));

        appointment_id
    }

    pub fn confirm_appointment(env: &Env, provider: Address, appointment_id: U256) {
        let mut appointments = Self::get_appointments(env);
        let mut appointment = appointments.get(appointment_id).unwrap_or_else(|| {
            panic_with_error!(env, "Appointment not found");
        });

        if appointment.provider != provider {
            panic_with_error!(env, "Only provider can confirm appointment");
        }

        if appointment.status != AppointmentStatus::Booked {
            panic_with_error!(env, "Appointment cannot be confirmed in current status");
        }

        appointment.status = AppointmentStatus::Confirmed;
        appointments.set(appointment_id, appointment);
        Self::set_appointments(env, &appointments);
    }

    pub fn complete_appointment(env: &Env, provider: Address, appointment_id: U256) {
        let mut appointments = Self::get_appointments(env);
        let mut appointment = appointments.get(appointment_id).unwrap_or_else(|| {
            panic_with_error!(env, "Appointment not found");
        });

        if appointment.provider != provider {
            panic_with_error!(env, "Only provider can complete appointment");
        }

        if appointment.status != AppointmentStatus::Confirmed {
            panic_with_error!(env, "Appointment must be confirmed before completion");
        }

        appointment.status = AppointmentStatus::Completed;
        appointments.set(appointment_id, appointment);
        Self::set_appointments(env, &appointments);

        // Transfer funds to provider (this would integrate with token contract)
        // For now, we just mark as completed
    }

    pub fn refund_appointment(env: &Env, student: Address, appointment_id: U256) {
        let mut appointments = Self::get_appointments(env);
        let mut appointment = appointments.get(appointment_id).unwrap_or_else(|| {
            panic_with_error!(env, "Appointment not found");
        });

        if appointment.student != student {
            panic_with_error!(env, "Only student can request refund");
        }

        if appointment.status != AppointmentStatus::Booked && appointment.status != AppointmentStatus::Confirmed {
            panic_with_error!(env, "Appointment cannot be refunded in current status");
        }

        appointment.status = AppointmentStatus::Refunded;
        appointments.set(appointment_id, appointment);
        Self::set_appointments(env, &appointments);

        // Refund funds to student (this would integrate with token contract)
        // For now, we just mark as refunded
    }

    pub fn cancel_appointment(env: &Env, caller: Address, appointment_id: U256) {
        let mut appointments = Self::get_appointments(env);
        let mut appointment = appointments.get(appointment_id).unwrap_or_else(|| {
            panic_with_error!(env, "Appointment not found");
        });

        if caller != appointment.student && caller != appointment.provider {
            panic_with_error!(env, "Only student or provider can cancel appointment");
        }

        if appointment.status == AppointmentStatus::Completed || appointment.status == AppointmentStatus::Refunded {
            panic_with_error!(env, "Cannot cancel completed or refunded appointment");
        }

        appointment.status = AppointmentStatus::Canceled;
        appointments.set(appointment_id, appointment);
        Self::set_appointments(env, &appointments);

        // Refund automatically on cancellation
        if caller == appointment.student {
            appointment.status = AppointmentStatus::Refunded;
            appointments.set(appointment_id, appointment);
            Self::set_appointments(env, &appointments);
        }
    }

    pub fn get_appointment(env: &Env, appointment_id: U256) -> Appointment {
        Self::get_appointments(env)
            .get(appointment_id)
            .unwrap_or_else(|| panic_with_error!(env, "Appointment not found"))
    }

    pub fn get_student_appointments(env: &Env, student: Address) -> Vec<U256> {
        Self::get_student_appointments(env)
            .get(student)
            .unwrap_or_else(|| Vec::new(env))
    }

    pub fn get_provider_appointments(env: &Env, provider: Address) -> Vec<U256> {
        Self::get_provider_appointments(env)
            .get(provider)
            .unwrap_or_else(|| Vec::new(env))
    }
}

#[contractimpl]
impl AppointmentEscrowContract {
    pub fn book_appointment(
        env: Env,
        student: Address,
        provider: Address,
        amount: U256,
        token_address: Address,
    ) -> U256 {
        AppointmentEscrowData::book_appointment(&env, student, provider, amount, token_address)
    }

    pub fn confirm_appointment(env: Env, provider: Address, appointment_id: U256) {
        AppointmentEscrowData::confirm_appointment(&env, provider, appointment_id);
    }

    pub fn complete_appointment(env: Env, provider: Address, appointment_id: U256) {
        AppointmentEscrowData::complete_appointment(&env, provider, appointment_id);
    }

    pub fn refund_appointment(env: Env, student: Address, appointment_id: U256) {
        AppointmentEscrowData::refund_appointment(&env, student, appointment_id);
    }

    pub fn cancel_appointment(env: Env, caller: Address, appointment_id: U256) {
        AppointmentEscrowData::cancel_appointment(&env, caller, appointment_id);
    }

    pub fn get_appointment(env: Env, appointment_id: U256) -> Appointment {
        AppointmentEscrowData::get_appointment(&env, appointment_id)
    }

    pub fn get_appointment_status(env: Env, appointment_id: U256) -> Symbol {
        let appointment = AppointmentEscrowData::get_appointment(&env, appointment_id);
        appointment.status.to_symbol()
    }

    pub fn get_student_appointments(env: Env, student: Address) -> Vec<U256> {
        AppointmentEscrowData::get_student_appointments(&env, student)
    }

    pub fn get_provider_appointments(env: Env, provider: Address) -> Vec<U256> {
        AppointmentEscrowData::get_provider_appointments(&env, provider)
    }
}
