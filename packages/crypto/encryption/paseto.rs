#[derive(Default)]
#[cfg_attr(feature = "dev", derive(Debug, PartialEq))]
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

#[test]
fn test_header() {
    let header = crate::header!("aud" => "https://example.com", "sub" => "http://example.com");

    println!("{:#?}", header);
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
