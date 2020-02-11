use super::Responder;
use crate::context::Context;

use async_trait::async_trait;
use hyper::{header, Body, Response};
use std::future::Future;

pub type ResponseResult = http::Result<Response<Body>>;

#[async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn call(&self, ctx: Context) -> ResponseResult;
}

#[async_trait]
impl<T, F> Handler for T
where
    T: Fn(Context) -> F + Send + Sync + 'static,
    F: Future + Send + 'static,
    F::Output: Responder,
{
    async fn call(&self, ctx: Context) -> ResponseResult {
        let response = (self)(ctx).await.respond_to();

        let mut res = Response::builder();
        if let Some(headers) = response.headers() {
            if let Some(response_headers) = res.headers_mut() {
                headers.iter().for_each(move |(key, value)| {
                    if let Ok(val_bytes) = header::HeaderValue::from_bytes(value.as_bytes()) {
                        response_headers.insert(key, val_bytes);
                    }
                });
            }
        }

        if let Some(cookies) = response.cookies() {
            if let Some(response_headers) = res.headers_mut() {
                cookies.iter().for_each(move |cookie| {
                    if let Ok(cookie_str) =
                        header::HeaderValue::from_bytes(cookie.to_string().as_bytes())
                    {
                        if response_headers.contains_key(header::SET_COOKIE) {
                            response_headers.append(header::SET_COOKIE, cookie_str);
                        }
                        else {
                            response_headers.insert(header::SET_COOKIE, cookie_str);
                        }
                    }
                });
            }
        }

        res.status(response.status()).body(response.body())
    }
}
