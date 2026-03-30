use teachlink_contract::{AppointmentEscrowContract, AppointmentStatus};
use soroban_sdk::{symbol_short, Address, Env, U256};

#[test]
fn test_appointment_booking() {
    let env = Env::default();
    let student = Address::generate(&env);
    let provider = Address::generate(&env);
    let token_address = Address::generate(&env);
    let amount = U256::from_u32(100);
    
    let appointment_id = AppointmentEscrowContract::book_appointment(
        env.clone(),
        student.clone(),
        provider.clone(),
        amount,
        token_address,
    );
    
    let appointment = AppointmentEscrowContract::get_appointment(env.clone(), appointment_id);
    assert_eq!(appointment.student, student);
    assert_eq!(appointment.provider, provider);
    assert_eq!(appointment.amount, amount);
    assert_eq!(appointment.status, AppointmentStatus::Booked);
}

#[test]
fn test_appointment_confirmation() {
    let env = Env::default();
    let student = Address::generate(&env);
    let provider = Address::generate(&env);
    let token_address = Address::generate(&env);
    let amount = U256::from_u32(100);
    
    let appointment_id = AppointmentEscrowContract::book_appointment(
        env.clone(),
        student.clone(),
        provider.clone(),
        amount,
        token_address,
    );
    
    AppointmentEscrowContract::confirm_appointment(env.clone(), provider.clone(), appointment_id);
    
    let status = AppointmentEscrowContract::get_appointment_status(env.clone(), appointment_id);
    assert_eq!(status, symbol_short!("Confirmed"));
}

#[test]
fn test_appointment_completion() {
    let env = Env::default();
    let student = Address::generate(&env);
    let provider = Address::generate(&env);
    let token_address = Address::generate(&env);
    let amount = U256::from_u32(100);
    
    let appointment_id = AppointmentEscrowContract::book_appointment(
        env.clone(),
        student.clone(),
        provider.clone(),
        amount,
        token_address,
    );
    
    AppointmentEscrowContract::confirm_appointment(env.clone(), provider.clone(), appointment_id);
    AppointmentEscrowContract::complete_appointment(env.clone(), provider.clone(), appointment_id);
    
    let status = AppointmentEscrowContract::get_appointment_status(env.clone(), appointment_id);
    assert_eq!(status, symbol_short!("Completed"));
}

#[test]
fn test_appointment_refund() {
    let env = Env::default();
    let student = Address::generate(&env);
    let provider = Address::generate(&env);
    let token_address = Address::generate(&env);
    let amount = U256::from_u32(100);
    
    let appointment_id = AppointmentEscrowContract::book_appointment(
        env.clone(),
        student.clone(),
        provider.clone(),
        amount,
        token_address,
    );
    
    AppointmentEscrowContract::refund_appointment(env.clone(), student.clone(), appointment_id);
    
    let status = AppointmentEscrowContract::get_appointment_status(env.clone(), appointment_id);
    assert_eq!(status, symbol_short!("Refunded"));
}

#[test]
fn test_appointment_cancellation() {
    let env = Env::default();
    let student = Address::generate(&env);
    let provider = Address::generate(&env);
    let token_address = Address::generate(&env);
    let amount = U256::from_u32(100);
    
    let appointment_id = AppointmentEscrowContract::book_appointment(
        env.clone(),
        student.clone(),
        provider.clone(),
        amount,
        token_address,
    );
    
    AppointmentEscrowContract::cancel_appointment(env.clone(), student.clone(), appointment_id);
    
    let status = AppointmentEscrowContract::get_appointment_status(env.clone(), appointment_id);
    assert_eq!(status, symbol_short!("Refunded")); // Cancellation by student triggers refund
}

#[test]
fn test_student_appointments() {
    let env = Env::default();
    let student = Address::generate(&env);
    let provider = Address::generate(&env);
    let token_address = Address::generate(&env);
    let amount = U256::from_u32(100);
    
    let appointment_id1 = AppointmentEscrowContract::book_appointment(
        env.clone(),
        student.clone(),
        provider.clone(),
        amount,
        token_address,
    );
    
    let appointment_id2 = AppointmentEscrowContract::book_appointment(
        env.clone(),
        student.clone(),
        provider.clone(),
        amount,
        token_address,
    );
    
    let student_appointments = AppointmentEscrowContract::get_student_appointments(env.clone(), student.clone());
    assert_eq!(student_appointments.len(), 2);
    assert!(student_appointments.contains(appointment_id1));
    assert!(student_appointments.contains(appointment_id2));
}

#[test]
fn test_provider_appointments() {
    let env = Env::default();
    let student = Address::generate(&env);
    let provider = Address::generate(&env);
    let token_address = Address::generate(&env);
    let amount = U256::from_u32(100);
    
    let appointment_id = AppointmentEscrowContract::book_appointment(
        env.clone(),
        student.clone(),
        provider.clone(),
        amount,
        token_address,
    );
    
    let provider_appointments = AppointmentEscrowContract::get_provider_appointments(env.clone(), provider.clone());
    assert_eq!(provider_appointments.len(), 1);
    assert!(provider_appointments.contains(appointment_id));
}

#[test]
#[should_panic(expected = "Only provider can confirm appointment")]
fn test_unauthorized_confirmation() {
    let env = Env::default();
    let student = Address::generate(&env);
    let provider = Address::generate(&env);
    let unauthorized_user = Address::generate(&env);
    let token_address = Address::generate(&env);
    let amount = U256::from_u32(100);
    
    let appointment_id = AppointmentEscrowContract::book_appointment(
        env.clone(),
        student.clone(),
        provider.clone(),
        amount,
        token_address,
    );
    
    // Try to confirm with unauthorized user
    AppointmentEscrowContract::confirm_appointment(env.clone(), unauthorized_user, appointment_id);
}
