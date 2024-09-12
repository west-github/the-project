use axum::{
    extract::FromRequestParts,
    http::{request::Parts, Request, StatusCode},
};
use base64::{
    engine::{
        general_purpose::{NO_PAD, PAD},
        GeneralPurpose,
    },
    DecodeError, Engine as _,
};
use std::{
    any::type_name,
    env,
    marker::PhantomData,
    str::FromStr,
    task::{Context, Poll},
};
use tower_layer::Layer;
use tower_service::Service;

//  Re - export
pub use derive_new::new;

pub fn get_env(name: &'static str) -> core::result::Result<String, String> {
    env::var(name).map_err(|_| f!("{} not found in environment", name))
}

pub fn get_env_parse<T: FromStr>(name: &'static str) -> core::result::Result<T, String> {
    let msg = f!("Failed to parse {} into {}", name, std::any::type_name::<T>());
    get_env(name).and_then(|value| value.parse::<T>().map_err(|_| msg))
}

#[macro_export]
macro_rules! lazy_lock {
    ($definition:expr) => {
        std::sync::LazyLock::new(|| $definition)
    };
    (() => $block:block) => {
        std::sync::LazyLock::new(|| $block)
    };
}

#[doc = "Return the error provided if the predicate is false"]
#[macro_export]
macro_rules! ensure {
    ($pred:expr,  $err:expr) => {
        if !$pred {
            return Err($err);
        }
    };
}

#[doc = "Return error always, this function short circuit"]
#[macro_export]
macro_rules! err {
    ($err:expr) => {
        return Err($err)
    };
}

#[macro_export]
macro_rules! lock {
    ($lock:expr) => {
        $lock.lock().unwrap()
    };
    ($lock:expr, $error:expr) => {{
        match $lock.lock() {
            Ok(lock) => lock,
            Err(_) => return $error,
        }
    }};
}

#[macro_export]
macro_rules! clone {
    ($expr:expr) => {
        $expr.clone()
    };
}

#[macro_export]
macro_rules! duration_since {
    ($earlier:expr) => {{
        std::time::Instant::now().duration_since($earlier)
    }};
}

#[macro_export]
macro_rules! f {
    ($($arg:tt)*) => {
        format!($($arg)*)
    };
}

#[macro_export]
macro_rules! impl_error_display {
    ($ident:ident) => {
        impl std::error::Error for $ident {}

        impl std::fmt::Display for $ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Error: {:?}", self)
            }
        }
    };
}

#[macro_export]
macro_rules! opt {
    ($( $value:expr )?) => {{
        match ($(Some($value))?) {$(| Some(_) => Some($value),)?| _ => None}
    }};
}

#[macro_export]
macro_rules! arc {
    ($value:expr) => {
        std::sync::Arc::new($value)
    };
}

#[macro_export]
macro_rules! mutex {
    ($value:expr) => {
        std::sync::Mutex::new($value)
    };
}

#[macro_export]
macro_rules! to_static {
    ($ty:ty, $data:expr) => {{
        static DATA: std::sync::LazyLock<$ty> = $crate::lazy_lock!($data);
        &*DATA
    }};
}

#[macro_export]
macro_rules! static_s {
    ($data:expr) => {{
        $crate::StaticLayer::new($data)
    }};
}

#[doc = r#"
```not_rust
string!() => Empty String
string!(content) => String with content
string!(u8: content) => String from u8
string!(u8l: content) => String from lossy U8 can fail
string!(u16: content) => String from u16
string!(u16l: content) => String from lossy u16 can fail
```
"#]
#[macro_export]
macro_rules! string {
    () => {
        String::new()
    };

    ($content:expr) => {
        String::from($content)
    };

    ($content:expr, $cap:expr) => {{
        let mut string = String::with_capacity($cap);
        string.push_str($content);
        string
    }};

    // From Methods
    (u8: $content:expr) => {
        String::from_utf8($content)
    };

    (u8l: $content:expr) => {
        String::from_utf8_lossy($content)
    };

    (u16: $content:expr) => {
        String::from_utf16($content)
    };

    (u16l: $content:expr) => {
        String::from_utf16_lossy($content)
    };
}

#[derive(new, Clone)]
pub struct AddStatic<S, T: 'static> {
    inner: S,
    ext: &'static T,
}

#[derive(new, Clone)]
pub struct StaticLayer<T: 'static> {
    ext: &'static T,
}

impl<S, T> Layer<S> for StaticLayer<T>
where
    T: Clone + 'static,
{
    type Service = AddStatic<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        AddStatic::new(inner, self.ext)
    }
}

#[derive(new, Clone)]
pub struct Static<T: 'static>(pub &'static T);

impl<T> std::ops::Deref for Static<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<ReqBody, S, T> Service<Request<ReqBody>> for AddStatic<S, T>
where
    S: Service<Request<ReqBody>>,
    Static<T>: Send + Sync + Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        req.extensions_mut().insert(Static::new(self.ext));
        self.inner.call(req)
    }
}

#[async_trait::async_trait]
impl<S, T> FromRequestParts<S> for Static<T>
where
    Static<T>: Send + Send + Sync + 'static + Clone,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        if let Some(value) = parts.extensions.get::<Static<T>>().cloned() {
            return Ok(value);
        }

        if cfg!(test) {
            panic!("Failed to  extract {}, is it added via StaticLayer", type_name::<Static<T>>())
        } else {
            tracing::error!("Failed to  extract {}, is it added via StaticLayer", type_name::<Static<T>>());
        }

        Err((StatusCode::INTERNAL_SERVER_ERROR, "Unknown error occurred!"))
    }
}

pub trait Encoding {
    const NAME: &'static str;
    type Success;
    type Error;

    fn encode(&self, input: impl AsRef<[u8]>) -> Result<Self::Success, Self::Error>;

    fn decode(&self, input: impl AsRef<[u8]>) -> Result<Self::Success, Self::Error>;
}

pub trait Encryption {
    type Success;
    type Error;
    type Claim;

    fn encrypt(&self, claim: Self::Claim) -> Result<Self::Success, Self::Error>;

    fn decrypt<T>(&self, content: Self::Success, claim: Self::Claim) -> Result<T, Self::Error>;
}

pub trait Hashing {
    type Error;

    fn hash(&self, content: &str) -> Result<String, Self::Error>;

    fn verify(&self, content: &str, other: &str) -> Result<bool, Self::Error>;
}

/// ```no_rust
/// Base64
///
/// Encode and Decode bytes using base64 encoding
/// ```
#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct B64<T = UrlSafe>(PhantomData<T>);

impl<T> B64<T> {
    pub fn new() -> Self {
        B64(PhantomData)
    }
}

const STANDARD: GeneralPurpose = GeneralPurpose::new(&base64::alphabet::STANDARD, PAD);
const STANDARD_NO_PAD: GeneralPurpose = GeneralPurpose::new(&base64::alphabet::STANDARD, NO_PAD);
const URL_SAFE: GeneralPurpose = GeneralPurpose::new(&base64::alphabet::URL_SAFE, PAD);
const URL_SAFE_NO_PAD: GeneralPurpose = GeneralPurpose::new(&base64::alphabet::URL_SAFE, NO_PAD);

// These are used to enforced the standard we want
macro_rules! impl_encoding {
    ($ident:ident, $alg:expr, $name:expr) => {
        #[derive(Clone)]
        pub struct $ident;
        impl Encoding for B64<$ident> {
            const NAME: &'static str = $name;
            type Success = String;
            type Error = Error;

            fn encode(&self, input: impl AsRef<[u8]>) -> Result<Self::Success, Self::Error> {
                Ok($alg.encode(input))
            }

            fn decode(&self, input: impl AsRef<[u8]>) -> Result<Self::Success, Self::Error> {
                $alg.decode(input)
                    .map(String::from_utf8)?
                    .map_err(|_| Error(String::from("Failed to convert from bytes to string!")))
            }
        }
    };
}

#[derive(Debug)]
pub struct Error(pub String);

impl From<DecodeError> for Error {
    fn from(value: DecodeError) -> Self {
        use base64::DecodeError::*;

        match value {
            InvalidByte(offset, bytes) => Error(f!("Invalid token byte at offset: {} bytes = {}", offset, bytes)),
            InvalidLength(length) => Error(f!("The length of the token is invalid length: {}", length)),
            InvalidLastSymbol(o, b) => Error(f!("Failed encoding, invalid offset: {} bytes = {}", o, b)),
            InvalidPadding => Error(string!("This token failed encoding to due to invalid padding")),
        }
    }
}

impl_encoding!(UrlSafe, URL_SAFE, "URLSAFE");
impl_encoding!(Standard, STANDARD, "STANDARD");
impl_encoding!(UrlSafeNopad, URL_SAFE_NO_PAD, "URLSAFE NOPAD");
impl_encoding!(StandardNopad, STANDARD_NO_PAD, "STANDARD NOPAD");
impl_error_display!(Error);

#[derive(Default)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Header<'a> {
    pub aud: Option<&'a str>,

    pub sub: Option<&'a str>,

    pub iss: Option<&'a str>,

    pub tid: Option<&'a str>,

    pub nbf: Option<&'a str>,

    pub iat: Option<&'a str>,

    pub exp: Option<&'a str>,

    /// Footer
    pub ftr: Option<&'a str>,

    /// Implicit assertions
    pub ixa: Option<&'a str>,
}

#[doc = r#"Construct header

aud = AudienceClaim

sub = SubjectClaim

iss = IssuerClaim

tid = TokenIdentificationClaim

nbf = Not Before claim

iat = IssuedAtClaim

exp = ExpirationClaim

ftr = FooterClaim

ixa = Implicit assertion claim
```rust
use lib_crypto::{header, Header};
let header = header!("aud" => "aud", "sub" => "sub", "iss" => "iss");
assert_eq!(header, Header{aud: Some("aud"), sub: Some("sub"), iss: Some("iss"), ..Default::default()});
```"#]
#[macro_export]
macro_rules! header {
    ($($ident:expr => $value:expr),+) => {{
        let mut header = $crate::Header::default();
        $(
            match $ident {
                "aud" => {header.aud = Some($value)},
                "sub" => {header.sub = Some($value)},
                "iss" => {header.iss = Some($value)},
                "tid" => {header.tid = Some($value)},
                "nbf" => {header.nbf = Some($value)},
                "iat" => {header.iat = Some($value)},
                "exp" => {header.exp = Some($value)},
                "ftr" => {header.ftr = Some($value)},
                "ixa" => {header.ixa = Some($value)},
                _ => {},
            }
        )+

        header
    }};
}

#[cfg(test)]
pub mod test {
    use std::sync::LazyLock;

    use crate::static_s;

    use super::*;
    use anyhow::{anyhow, Result};
    use axum::http::{Request, Response};
    use bytes::Bytes;
    use derive_new::new;
    use http_body_util::BodyExt;
    use tower::BoxError;
    use tower::{service_fn, ServiceBuilder, ServiceExt};

    type BoxBody = http_body_util::combinators::UnsyncBoxBody<Bytes, BoxError>;

    #[allow(dead_code)]
    pub struct Body(BoxBody);

    impl Body {
        pub fn new<B>(body: B) -> Self
        where
            B: http_body::Body<Data = Bytes> + Send + 'static,
            B::Error: Into<BoxError>,
        {
            Self(body.map_err(Into::into).boxed_unsync())
        }

        pub fn empty() -> Self {
            Self::new(http_body_util::Empty::new())
        }
    }

    #[derive(Debug, new, Clone)]
    struct Data(&'static str);

    #[tokio::test]
    async fn static_service() -> Result<()> {
        async fn handler(req: Request<Body>) -> Result<Response<String>> {
            fn extractor<T>(req: &Request<Body>) -> Result<&'static T>
            where
                T: Send + Sync + 'static,
            {
                let Static(_ext) = req
                    .extensions()
                    .get::<Static<T>>()
                    .ok_or(anyhow!(f!("Failed to extract: {}", std::any::type_name::<T>())))?;

                Ok(*_ext)
            }

            let Data(data) = extractor::<Data>(&req)?;
            let encoder = extractor::<B64<Standard>>(&req)?;
            Ok(Response::new(encoder.encode(data)?))
        }

        static ENCODER: LazyLock<B64<Standard>> = lazy_lock!(B64::<Standard>::new());
        let data: &'static Data = to_static!(Data, Data::new("West"));

        let res = ServiceBuilder::new()
            .layer(static_s!(data))
            .layer(static_s!(&*ENCODER))
            .service(service_fn(handler))
            .oneshot(Request::new(Body::empty()))
            .await?
            .into_body();

        let res = ENCODER.decode(res)?;

        assert_eq!("West", res);
        Ok(())
    }

    #[test]
    fn test_header() {
        let header = crate::header!("aud" => "https://example.com", "sub" => "http://example.com");

        println!("{:#?}", header);
    }

    /// Encode and Decode The value Passed testing the engine passed
    ///
    /// # Errors
    ///
    /// This function will return an error if encoding and decoding failed.
    fn encode_and_decode_handler<T>(engine: T, value: impl AsRef<[u8]>) -> Result<()>
    where
        T: Encoding<Success = String, Error = Error>,
    {
        let enc_content = engine.encode(value)?;

        println!("{} - {:?}", T::NAME, enc_content);
        println!("{} - {:?}", T::NAME, engine.decode(enc_content)?);
        Ok(())
    }

    #[test]
    fn test_standard_no_pad() -> Result<()> {
        encode_and_decode_handler(B64::<StandardNopad>::new(), "ABCDGETAJHE")
    }

    #[test]
    fn test_url_safe_no_pad() -> Result<()> {
        encode_and_decode_handler(B64::<UrlSafeNopad>::new(), "ABCDGETAJHE")
    }

    #[test]
    fn test_standard() -> Result<()> {
        encode_and_decode_handler(B64::<Standard>::new(), "ABCDGETAJHE")
    }

    #[test]
    fn test_url_safe() -> Result<()> {
        encode_and_decode_handler(B64::<UrlSafe>::new(), "ABCDGETAJHE")
    }
}
