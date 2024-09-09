#[cfg(feature = "bearer")]
pub mod bearer;
#[cfg(feature = "cookie")]
pub mod cookie;
#[cfg(feature = "jwt")]
pub mod jwt;
#[cfg(feature = "oauth")]
pub mod oauth;

//
pub mod u_auth;

use async_trait::async_trait;
use derive_new::new;
use http::{Request, Uri};
use std::collections::HashMap;
use std::task::{Context, Poll};
use tower_layer::Layer;
use tower_service::Service;

#[derive(new)]
#[cfg_attr(feature = "dev", derive(Debug))]
pub struct Success<C> {
    claims: C,
    state: State,
}

#[derive(new)]
#[cfg_attr(feature = "dev", derive(Debug))]
pub struct Failure<C, E> {
    state: State,
    claims: C,
    help: E,
}

#[async_trait]
pub trait Authentication<Request> {
    type Claim;
    type Error;

    // This is useful for error handling
    const NAME: &'static str;

    /// Authenticate the current Request
    async fn authenticate(&self, req: Request) -> Result<Success<Self::Claim>, Failure<Self::Claim, Self::Error>>;

    //  We might need to return something that can be turn to response cause we are not mutating response
    /// Forbid the current request
    async fn forbid(&self, req: Request, state: State);

    /// Challenge the current request
    async fn challenge(&self, req: Request, state: State);

    /// Sign in the current Request
    async fn sign_in(&self, req: Request, claims: Self::Claim, state: State) -> Result<Success<Self::Claim>, Failure<Self::Claim, Self::Error>>;

    /// Sign out the current Request
    async fn sign_out(&self, req: Request, claims: Self::Claim, state: State) -> Result<Success<Self::Claim>, Failure<Self::Claim, Self::Error>>;
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub allow_refresh: bool,
    pub is_persistent: bool,
    pub redirect_url: Uri,

    // Store unnamed state
    _state: HashMap<String, String>,
    //
}

impl State {
    pub fn set_url(&mut self, url: Uri) -> &mut Self {
        self.redirect_url = url;
        self
    }

    //  Return Self allow chaining and setting multiple values at once
    pub fn set_state(&mut self, key: &str, value: &str) -> &mut Self {
        self._state.insert(key.into(), value.into());
        self
    }

    pub fn get_state(&self, key: &str) -> Option<&str> {
        self._state.get(key).as_ref()
    }

    pub fn set_all<'a, T>(&mut self, values: T) -> &mut Self
    where
        T: IntoIterator<Item = (&'a str, &'a str)>,
    {
        for (key, value) in values.into_iter() {
            self.set_state(key, value);
        }

        self
    }
}

impl From<&Uri> for State {
    fn from(value: &Uri) -> Self {
        let redirect_url = value.to_owned();

        State {
            redirect_url,
            ..State::default()
        }
    }
}

#[derive(new)]
pub struct AddStaticExtensions<S, T: 'static> {
    inner: S,
    ext: Vec<&'static T>,
}

impl<ReqBody, S, T> Service<Request<ReqBody>> for AddStaticExtensions<S, T>
where
    S: Service<Request<ReqBody>>,
    T: Send + Sync + 'static + Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        for _s in &self.ext {
            req.extensions_mut().insert((*_s).clone());
        }

        self.inner.call(req)
    }
}

#[derive(new)]
pub struct StaticExtensionsLayer<T: 'static> {
    services: Vec<&'static T>,
}

impl<S, T> Layer<S> for StaticExtensionsLayer<T>
where
    T: Clone,
{
    type Service = AddStaticExtensions<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        AddStaticExtensions::new(inner, self.services.clone())
    }
}

#[derive(new)]
pub struct AuthenticationService<S, T> {
    inner: S,
    // Other default service will be added later
    state: State,
    auth_extensions: Vec<T>,
}

pub struct AuthenticationLayer<T> {
    // Other default service will be added later
    state: State,
    auth_extensions: Vec<T>,
}

impl<S, T, ReqBody> Service<Request<ReqBody>> for AuthenticationService<S, T>
where
    S: Service<Request<ReqBody>>,
    T: Authentication<Request<ReqBody>> + Clone + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let ext = req.extensions_mut();
        ext.insert(self.state.clone());

        while let Some(service) = self.auth_extensions.iter().next() {
            // T::NAME;

            ext.insert(service.clone());
        }

        self.inner.call(req)
    }
}

impl<S, T> Layer<S> for AuthenticationLayer<T>
where
    T: Clone,
{
    type Service = AuthenticationService<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthenticationService::new(inner, self.state.clone(), self.auth_extensions.clone())
    }
}

#[cfg(test)]
pub mod test {
    use super::State;
    use super::StaticExtensionsLayer;
    use bytes::Bytes;
    use derive_new::new;
    use http::Uri;
    use http::{Request, Response};
    use http_body_util::BodyExt;
    use std::{convert::Infallible, sync::LazyLock};
    use tower::BoxError;
    use tower::{service_fn, ServiceBuilder, ServiceExt};

    type BoxBody = http_body_util::combinators::UnsyncBoxBody<Bytes, BoxError>;

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

    #[test]
    fn state() {
        let mut state = State::default();

        state.set_all([("Login", "true")]);

        if let Some(value) = state.get_state("Login") {
            assert_eq!(value, "true");
        }
    }

    #[test]
    fn from_uri() -> anyhow::Result<()> {
        let url = "http://localhost:3000".parse::<Uri>()?;

        let mut state = State::from(&url);
        state.set_state("is_created", "true");
        println!("{:#?}", state);

        if let Some(value) = state.get_state("is_crated") {
            assert_eq!(value, "true");
        }

        Ok(())
    }

    #[derive(Debug, new, Clone)]
    struct Data {
        content: String,
    }

    static DATA: LazyLock<Data> = LazyLock::new(|| Data::new(String::from("West")));

    #[tokio::test]
    async fn static_service() -> anyhow::Result<()> {
        async fn handler(req: Request<Body>) -> Result<Response<String>, Infallible> {
            let state = req.extensions().get::<Data>().unwrap();

            Ok(Response::new(state.content.clone()))
        }
        let extensions = vec![&*DATA];

        let svc = ServiceBuilder::new()
            .layer(StaticExtensionsLayer::new(extensions))
            .service(service_fn(handler));

        let res: String = svc.oneshot(Request::new(Body::empty())).await?.into_body();

        println!("{}", res);
        assert_eq!(String::from("West"), res);
        Ok(())
    }
}
