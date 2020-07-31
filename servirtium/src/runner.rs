use crate::{error::Error, servirtium_server::RequestData, ServirtiumServer, TestSession};
use hyper::{
    body,
    header::{HeaderName, HeaderValue},
    service::{make_service_fn, service_fn},
    Body, HeaderMap, Request, Response, Server,
};
use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Once, thread};
use tokio::runtime::Runtime;

static INITIALIZE_SERVIRTIUM: Once = Once::new();

pub(crate) fn start_once() {
    INITIALIZE_SERVIRTIUM.call_once(|| {
        let mut server_instance = ServirtiumServer::instance();

        server_instance.join_handle = Some(thread::spawn(move || {
            Runtime::new().unwrap().block_on(async {
                let addr = SocketAddr::from(([127, 0, 0, 1], 61417));

                let server = Server::bind(&addr).serve(make_service_fn(|_| async {
                    Ok::<_, Infallible>(service_fn(|req| async move {
                        match handle_request(req).await {
                            Ok(response) => Ok(response),
                            Err(err) => {
                                TestSession::set_error(err);
                                Ok::<Response<Body>, Infallible>(Response::new(Body::empty()))
                            }
                        }
                    }))
                }));

                if let Err(e) = server.await {
                    eprintln!("Servirtium Server error: {}", e);
                }
            });
        }));

        server_instance.release_instance();
    });
}

async fn handle_request(mut request: Request<Body>) -> Result<Response<Body>, Error> {
    let mut instance = ServirtiumServer::instance();
    let request_data = read_request_data(&mut request).await?;

    let response_data = instance.handle_request(request_data).await?;
    instance.release_instance();

    let mut response_builder = Response::builder().status(response_data.status_code);

    put_headers(
        response_builder.headers_mut().ok_or(Error::InvalidBody)?,
        &response_data.headers,
    )?;

    Ok(response_builder.body(response_data.body.into())?)
}

async fn read_request_data(request: &mut Request<Body>) -> Result<RequestData, Error> {
    let method = request.method().to_string();
    let uri = request.uri().to_string();
    let headers = extract_headers(request.headers());

    let body = body::to_bytes(request.body_mut())
        .await
        .map_err(|_| Error::InvalidBody)?;

    Ok(RequestData {
        method,
        uri,
        headers,
        body: String::from_utf8_lossy(&body).into(),
    })
}

fn extract_headers(header_map: &HeaderMap) -> HashMap<String, String> {
    // it currently ignores header values with opaque characters
    header_map
        .iter()
        .map(|(k, v)| (String::from(k.as_str()), v.to_str()))
        .filter_map(|(key, value)| value.ok().map(|v| (key, String::from(v))))
        .collect::<HashMap<_, _>>()
}

fn put_headers<'a, I: IntoIterator<Item = (&'a String, &'a String)>>(
    header_map: &mut HeaderMap<HeaderValue>,
    headers: I,
) -> Result<(), Error> {
    for (key, value) in headers {
        let header_name = HeaderName::from_lowercase(key.to_lowercase().as_bytes())?;
        let header_value = HeaderValue::from_str(value)?;
        header_map.append(header_name, header_value);
    }

    Ok(())
}
