use soroban_sdk::Address;

#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    WrappedToken,
    TotalWrapped,
}
