use teachlink_contract::{RbacContract, Role};
use soroban_sdk::{symbol_short, Address, Env, Symbol};

#[test]
fn test_rbac_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);
    
    RbacContract::initialize(env.clone(), admin.clone());
    
    assert_eq!(RbacContract::get_admin(env.clone()), admin);
}

#[test]
fn test_role_assignment() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    
    RbacContract::initialize(env.clone(), admin.clone());
    
    // Assign Doctor role
    RbacContract::assign_role(
        env.clone(),
        admin.clone(),
        user.clone(),
        symbol_short!("Doctor"),
    );
    
    assert!(RbacContract::has_role(
        env.clone(),
        user.clone(),
        symbol_short!("Doctor")
    ));
    
    // Check user roles
    let user_roles = RbacContract::get_user_roles(env.clone(), user.clone());
    assert_eq!(user_roles.len(), 1);
    assert_eq!(user_roles.get(0).unwrap(), symbol_short!("Doctor"));
}

#[test]
fn test_role_removal() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    
    RbacContract::initialize(env.clone(), admin.clone());
    
    // Assign and then remove Doctor role
    RbacContract::assign_role(
        env.clone(),
        admin.clone(),
        user.clone(),
        symbol_short!("Doctor"),
    );
    
    RbacContract::remove_role(
        env.clone(),
        admin.clone(),
        user.clone(),
        symbol_short!("Doctor"),
    );
    
    assert!(!RbacContract::has_role(
        env.clone(),
        user.clone(),
        symbol_short!("Doctor")
    ));
}

#[test]
fn test_multiple_roles() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    
    RbacContract::initialize(env.clone(), admin.clone());
    
    // Assign multiple roles
    RbacContract::assign_role(
        env.clone(),
        admin.clone(),
        user.clone(),
        symbol_short!("Doctor"),
    );
    RbacContract::assign_role(
        env.clone(),
        admin.clone(),
        user.clone(),
        symbol_short!("Patient"),
    );
    
    assert!(RbacContract::has_role(
        env.clone(),
        user.clone(),
        symbol_short!("Doctor")
    ));
    assert!(RbacContract::has_role(
        env.clone(),
        user.clone(),
        symbol_short!("Patient")
    ));
    
    let user_roles = RbacContract::get_user_roles(env.clone(), user.clone());
    assert_eq!(user_roles.len(), 2);
}

#[test]
#[should_panic(expected = "Only admin can assign roles")]
fn test_unauthorized_role_assignment() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let unauthorized_user = Address::generate(&env);
    let target_user = Address::generate(&env);
    
    RbacContract::initialize(env.clone(), admin.clone());
    
    // Try to assign role without admin privileges
    RbacContract::assign_role(
        env.clone(),
        unauthorized_user,
        target_user,
        symbol_short!("Doctor"),
    );
}
