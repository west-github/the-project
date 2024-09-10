use anyhow::Result;
use lib_crypto::*;

/// Encode and Decode The value Passed testing the engine passed
///
/// # Errors
///
/// This function will return an error if encoding and decoding failed.
fn encode_and_decode_handler<T>(engine: &T, value: impl AsRef<[u8]>, msg: &str) -> Result<()>
where
    T: Encoding<Success = String, Error = Error>,
{
    use lib_crypto::{decode, encode};
    let enc_content = encode(engine, value)?;

    println!("{} - {:?}", msg, enc_content);
    println!("{} - {:?}", msg, decode(engine, enc_content)?);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::encode_and_decode_handler;
    use anyhow::Result;
    use lib_crypto::*;

    #[test]
    fn test_standard() -> Result<()> {
        encode_and_decode_handler(&B64::<Standard>::new(), "ABCDGETAJHE", "BASE64 - STANDARD")
    }

    #[test]
    fn test_standard_no_pad() -> Result<()> {
        encode_and_decode_handler(&B64::<StandardNopad>::new(), "ABCDGETAJHE", "BASE64 - STANDARD NOPAD")
    }

    #[test]
    fn test_url_safe() -> Result<()> {
        encode_and_decode_handler(&B64::<UrlSafe>::new(), "ABCDGETAJHE", "BASE64 - URLSAFE")
    }

    #[test]
    fn test_url_safe_no_pad() -> Result<()> {
        encode_and_decode_handler(&B64::<UrlSafeNopad>::new(), "ABCDGETAJHE", "BASE64 - URLSAFE NOPAD")
    }
}
