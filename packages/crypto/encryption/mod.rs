pub(crate) mod paseto;

pub trait Encryption {
    type Success;
    type Error;
    type Claim;

    fn encrypt(&self, claim: Self::Claim) -> Result<Self::Success, Self::Error>;

    fn decrypt<T>(&self, content: Self::Success, claim: Self::Claim) -> Result<T, Self::Error>;
}
