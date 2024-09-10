use axum::{
    extract::FromRequestParts,
    http::{request::Parts, Request, StatusCode},
};
use std::{
    any::type_name,
    env,
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
macro_rules! __impl_error_display {
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
macro_rules! static_s {
    ($ty:ident, $data:expr) => {{
        static DATA: std::sync::LazyLock<$ty> = $crate::lazy_lock!($data);
        $crate::StaticLayer::new(&*DATA)
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

#[cfg(test)]
pub mod test {
    use super::*;
    use axum::http::{Request, Response};
    use bytes::Bytes;
    use derive_new::new;
    use http_body_util::BodyExt;
    use std::convert::Infallible;
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
    async fn static_service() -> anyhow::Result<()> {
        async fn handler(req: Request<Body>) -> Result<Response<&'static str>, Infallible> {
            let static_data = req.extensions().get::<Static<Data>>().unwrap();
            Ok(Response::new(static_data.0 .0))
        }

        let res = ServiceBuilder::new()
            .layer(crate::static_s!(Data, Data::new("West")))
            .service(service_fn(handler))
            .oneshot(Request::new(Body::empty()))
            .await?
            .into_body();

        assert_eq!("West", res);

        println!("{}", res);
        Ok(())
    }
}
