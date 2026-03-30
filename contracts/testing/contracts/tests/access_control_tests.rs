#[test]
fn test_admin_can_assign_roles() {
    let mut ac = AccessControl::new(admin);
    ac.assign_role(user, Role::Governor);
    assert_eq!(ac.get_role(user), Some(Role::Governor));
}

#[test]
#[should_panic(expected = "Access denied")]
fn test_non_admin_cannot_assign_roles() {
    let mut ac = AccessControl::new(admin);
    ac.assign_role(user, Role::Governor); // should fail if caller != admin
}
