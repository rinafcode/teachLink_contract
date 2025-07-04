#[interface]
trait ITeachLinkStaking {
    fn stake(amount: u128, duration: u64);
    fn withdraw();
    fn emergency_withdraw();
}
