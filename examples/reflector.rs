extern crate hyper;
extern crate mime;
extern crate futures;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate fern;
#[macro_use]
extern crate log;
extern crate chrono;

extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate shib_gotham;

use log::LogLevelFilter;
use hyper::{Response, StatusCode};
use hyper::server::Http;
use gotham::handler::NewHandlerService;
use gotham::middleware::pipeline::new_pipeline;
use gotham::http::response::create_response;
use gotham::router::Router;
use gotham::router::request::path::NoopPathExtractor;
use gotham::router::request::query_string::NoopQueryStringExtractor;
use gotham::router::route::{RouteImpl, Extractors, Delegation};
use gotham::router::route::dispatch::{DispatcherImpl, new_pipeline_set, finalize_pipeline_set};
use gotham::router::route::matcher::any::AnyRouteMatcher;
use gotham::router::tree::TreeBuilder;
use gotham::router::tree::node::{SegmentType, NodeBuilder};
use gotham::router::response::finalizer::ResponseFinalizerBuilder;
use gotham::state::State;
use shib_gotham::{Shibbleware, ReceiverFailed};

fn main() {
    set_logging();
    let addr = "127.0.0.1:7878".parse().unwrap();

    let server = Http::new()
        .bind(&addr, NewHandlerService::new(router()))
        .unwrap();

    println!("Listening on http://{}", server.local_addr().unwrap());
    server.run().unwrap();
}

fn set_logging() {
    fern::Dispatch::new()
        .level(LogLevelFilter::Error)
        .level_for("gotham", log::LogLevelFilter::Trace)
        .level_for("gotham::state", log::LogLevelFilter::Error)
        .level_for("todo_session", log::LogLevelFilter::Error)
        .chain(std::io::stdout())
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}]{}",
                chrono::Utc::now().format("[%Y-%m-%d %H:%M:%S%.9f]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .apply()
        .unwrap();
}

#[derive(Serialize, Deserialize)]
struct Session {
    user: UserAttributes,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserAttributes {
    #[serde(rename = "User-Agent")]
    user_agent: String,

    #[serde(rename = "Accept")]
    accept: String,
}

mod controller {
    use super::*;

    pub fn welcome(state: State) -> (State, Response) {
        let body = br#"
            <html>
                <head>
                    <title>shib-gotham - Attribute Reflector Example</title>
                </head>
                <body>
                    <h2>Welcome</h2>
                    <p><a href="/auth/login">Login</a></p>
                </body>
            </html>
        "#;

        let response = create_response(
            &state,
            StatusCode::Ok,
            Some((body.to_vec(), mime::TEXT_HTML)),
        );

        (state, response)
    }
}

fn receive_subject(state: &mut State, attributes: UserAttributes) -> Result<(), ReceiverFailed> {
    println!("received attributes: {:?}", attributes);
    Ok(())
}

fn router() -> Router {
    let pipelines = new_pipeline_set();
    let (pipelines, protected) = pipelines.add(new_pipeline().add(Shibbleware::<Session>::new()));
    let pipelines = finalize_pipeline_set(pipelines);

    let default_pipeline_chain = ();
    let protected_pipeline_chain = (protected, ());

    let mut tree_builder = TreeBuilder::new();

    let welcome_route = {
        let dispatcher = DispatcherImpl::new(
            || Ok(controller::welcome),
            default_pipeline_chain,
            pipelines.clone(),
        );

        RouteImpl::new(
            AnyRouteMatcher::new(),
            Box::new(dispatcher),
            Extractors::<NoopPathExtractor, NoopQueryStringExtractor>::new(),
            Delegation::Internal,
        )
    };

    tree_builder.add_route(Box::new(welcome_route));

    let mut auth = NodeBuilder::new("auth", SegmentType::Static);

    let shib_route = {
        let dispatcher = DispatcherImpl::new(
            shib_gotham::auth_router(receive_subject),
            default_pipeline_chain,
            pipelines.clone(),
        );

        RouteImpl::new(
            AnyRouteMatcher::new(),
            Box::new(dispatcher),
            Extractors::<NoopPathExtractor, NoopQueryStringExtractor>::new(),
            Delegation::External,
        )
    };
    auth.add_route(Box::new(shib_route));

    tree_builder.add_child(auth);

    let tree = tree_builder.finalize();
    let response_finalizer = ResponseFinalizerBuilder::new().finalize();

    Router::new(tree, response_finalizer)
}
