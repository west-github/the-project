pub trait Hashing {
    type Error;

    fn hash(&self, content: &str) -> Result<String, Self::Error>;

    fn verify(&self, content: &str, other: &str) -> Result<bool, Self::Error>;
}

pub fn hash<T>(algs: T, content: &str) -> Result<String, T::Error>
where
    T: Hashing,
{
    algs.hash(content)
}

pub fn verify_hash<T>(algs: T, content: &str, other: &str) -> Result<bool, T::Error>
where
    T: Hashing,
{
    algs.verify(content, other)
}
