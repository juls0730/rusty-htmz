use hyped::*;
use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use http_body_util::{combinators::BoxBody, BodyExt, Empty};
use hyper::{body::Body, Method, StatusCode};

fn render_to_string(element: (Element, Element)) -> String {
    render((
        base().target("htmz"),
        doctype(),
        html((
            head((title("title"), meta().charset("utf-8"), style("body { font-family: Helvetica, Arial, Sans-Serif; background-color: #040506; color: #FAFAFA; } nav.nav { display: flex; flex-direction: row; } nav.nav a { margin-right: 6px; } "))),
            body((element.0, element.1, iframe().hidden().onload("setTimeout(()=>document.querySelector(contentWindow.location.hash||null)?.replaceWith(...contentDocument.body.childNodes))").name("htmz")))
        ))
    ))
}

async fn dashboard(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    // example for auth
    if let Some(cookie_header) = req.headers().get("Cookie") {
        if cookie_header.to_str().unwrap().contains("foo=") {
            let body = Full::new(Bytes::from(render(hyped::main("dashboard").id("main-content"))));
            
            return Ok(
                Response::builder().header("Content-Type", "text/html")
                .body(body.into())
                .unwrap()
            );
        }
    }
        
    let body = Full::new(Bytes::from(render(hyped::main("not authorized").id("main-content"))));

    Ok(
        Response::builder().header("Content-Type", "text/html")
        .body(body.into())
        .unwrap()
    )
}

async fn settings(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let body = Full::new(Bytes::from(render(hyped::main("settings").id("main-content"))));

    Ok(
        Response::builder().header("Content-Type", "text/html")
        .body(body.into())
        .unwrap()
    )
}

async fn index(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let body = Full::new(Bytes::from(render_to_string((
        div(
            nav(
                (
                    a("Dashboard").href("dashboard#main-content"),
                    a("Settings").href("settings#main-content")
                )
            )
            .class("nav")
        ),
        hyped::main("").id("main-content")
    ))));

    Ok(
        Response::builder().header("Content-Type", "text/html")
        .body(body.into())
        .unwrap()
    )
}

fn empty() -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("404 not found"))))
}

async fn router(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => Ok(index(req).await?),

        // Simply echo the body back to the client.
        (&Method::GET, "/dashboard") => Ok(dashboard(req).await?),

        // Convert to uppercase before sending back to client using a stream.
        (&Method::GET, "/settings") => Ok(settings(req).await?),

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = empty()?;
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(router))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
