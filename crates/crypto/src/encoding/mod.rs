pub(crate) mod base64;

pub trait Encoding {
    type Success;
    type Error;

    fn encode(&self, input: impl AsRef<[u8]>) -> Result<Self::Success, Self::Error>;

    fn decode(&self, input: impl AsRef<[u8]>) -> Result<Self::Success, Self::Error>;
}

pub fn encode<T>(enc: &T, input: impl AsRef<[u8]>) -> Result<T::Success, T::Error>
where
    T: Encoding,
{
    enc.encode(input)
}

pub fn decode<T>(dec: &T, input: impl AsRef<[u8]>) -> Result<T::Success, T::Error>
where
    T: Encoding,
{
    dec.decode(input)
}
