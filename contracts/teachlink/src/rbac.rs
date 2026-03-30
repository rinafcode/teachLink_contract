use soroban_sdk::{contract, contractimpl, Address, Env, Map, Symbol, Vec, panic_with_error};

#[contract]
pub struct RbacContract;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Admin,
    Doctor,
    Patient,
}

impl Role {
    pub fn to_symbol(&self) -> Symbol {
        match self {
            Role::Admin => Symbol::new(&Env::default(), "Admin"),
            Role::Doctor => Symbol::new(&Env::default(), "Doctor"),
            Role::Patient => Symbol::new(&Env::default(), "Patient"),
        }
    }

    pub fn from_symbol(env: &Env, symbol: &Symbol) -> Role {
        if *symbol == Symbol::new(env, "Admin") {
            Role::Admin
        } else if *symbol == Symbol::new(env, "Doctor") {
            Role::Doctor
        } else if *symbol == Symbol::new(env, "Patient") {
            Role::Patient
        } else {
            panic_with_error!(env, "Invalid role");
        }
    }
}

pub struct RbacData {
    pub roles: Map<Address, Vec<Role>>,
    pub admin: Address,
}

impl RbacData {
    const ROLES: Symbol = Symbol::new(&Env::default(), "ROLES");
    const ADMIN: Symbol = Symbol::new(&Env::default(), "ADMIN");

    pub fn new(env: &Env) -> Self {
        Self {
            roles: Map::new(env),
            admin: Address::generate(env),
        }
    }

    pub fn initialize(env: &Env, admin: Address) {
        let mut data = Self::new(env);
        data.admin = admin.clone();
        
        env.storage().instance().set(&Self::ADMIN, &admin);
        env.storage().instance().set(&Self::ROLES, &data.roles);
    }

    fn get_roles(env: &Env) -> Map<Address, Vec<Role>> {
        env.storage()
            .instance()
            .get(&Self::ROLES)
            .unwrap_or_else(|| Map::new(env))
    }

    fn set_roles(env: &Env, roles: &Map<Address, Vec<Role>>) {
        env.storage().instance().set(&Self::ROLES, roles);
    }

    fn get_admin(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&Self::ADMIN)
            .unwrap()
    }

    pub fn assign_role(env: &Env, caller: Address, user: Address, role: Role) {
        let admin = Self::get_admin(env);
        if caller != admin {
            panic_with_error!(env, "Only admin can assign roles");
        }

        let mut roles = Self::get_roles(env);
        let mut user_roles = roles.get(user.clone()).unwrap_or_else(|| Vec::new(env));
        
        if !user_roles.contains(&role) {
            user_roles.push_back(role);
            roles.set(user, user_roles);
            Self::set_roles(env, &roles);
        }
    }

    pub fn remove_role(env: &Env, caller: Address, user: Address, role: Role) {
        let admin = Self::get_admin(env);
        if caller != admin {
            panic_with_error!(env, "Only admin can remove roles");
        }

        let mut roles = Self::get_roles(env);
        if let Some(mut user_roles) = roles.get(user.clone()) {
            let mut index = None;
            for (i, user_role) in user_roles.iter().enumerate() {
                if user_role == role {
                    index = Some(i);
                    break;
                }
            }
            
            if let Some(idx) = index {
                user_roles.remove(idx as u32);
                if user_roles.is_empty() {
                    roles.remove(user);
                } else {
                    roles.set(user, user_roles);
                }
                Self::set_roles(env, &roles);
            }
        }
    }

    pub fn has_role(env: &Env, user: Address, role: Role) -> bool {
        let roles = Self::get_roles(env);
        if let Some(user_roles) = roles.get(user) {
            user_roles.contains(&role)
        } else {
            false
        }
    }

    pub fn get_user_roles(env: &Env, user: Address) -> Vec<Role> {
        let roles = Self::get_roles(env);
        roles.get(user).unwrap_or_else(|| Vec::new(env))
    }

    pub fn require_role(env: &Env, caller: Address, required_role: Role) {
        if !Self::has_role(env, caller, required_role) {
            panic_with_error!(env, "Access denied: required role not found");
        }
    }

    pub fn require_admin(env: &Env, caller: Address) {
        Self::require_role(env, caller, Role::Admin);
    }
}

#[contractimpl]
impl RbacContract {
    pub fn initialize(env: Env, admin: Address) {
        RbacData::initialize(&env, admin);
    }

    pub fn assign_role(env: Env, caller: Address, user: Address, role: Symbol) {
        let role_enum = Role::from_symbol(&env, &role);
        RbacData::assign_role(&env, caller, user, role_enum);
    }

    pub fn remove_role(env: Env, caller: Address, user: Address, role: Symbol) {
        let role_enum = Role::from_symbol(&env, &role);
        RbacData::remove_role(&env, caller, user, role_enum);
    }

    pub fn has_role(env: Env, user: Address, role: Symbol) -> bool {
        let role_enum = Role::from_symbol(&env, &role);
        RbacData::has_role(&env, user, role_enum)
    }

    pub fn get_user_roles(env: Env, user: Address) -> Vec<Symbol> {
        let roles = RbacData::get_user_roles(&env, user);
        roles.iter().map(|role| role.to_symbol()).collect()
    }

    pub fn get_admin(env: Env) -> Address {
        RbacData::get_admin(&env)
    }
}
