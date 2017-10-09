use std::fmt::Debug;

use serde::Deserialize;

use gotham::router::Router;
use gotham::router::tree::TreeBuilder;
use gotham::router::tree::node::{SegmentType, NodeBuilder};
use gotham::router::response::finalizer::ResponseFinalizerBuilder;
use gotham::router::request::path::NoopPathExtractor;
use gotham::router::route::{RouteImpl, Extractors, Delegation};
use gotham::router::route::matcher::any::AnyRouteMatcher;
use gotham::router::route::dispatch::{DispatcherImpl, new_pipeline_set, finalize_pipeline_set};

use receiver::{Receiver, LoginHandler, ReturnInfo};

/// Builds the subrouter for the Shibboleth-protected part of application, where new sessions will
/// be received for processing.
pub fn auth_router<A, R>(r: R) -> Router
where
    A: for<'de> Deserialize<'de> + Debug + 'static,
    R: Receiver<A> + Copy + 'static,
{
    let pipelines = finalize_pipeline_set(new_pipeline_set());

    let mut tree_builder = TreeBuilder::new();

    let mut node_builder = NodeBuilder::new("login", SegmentType::Static);

    let login_route = {
        let dispatcher =
            DispatcherImpl::new(move || Ok(LoginHandler::new(r)), (), pipelines.clone());

        RouteImpl::new(
            AnyRouteMatcher::new(),
            Box::new(dispatcher),
            Extractors::<NoopPathExtractor, ReturnInfo>::new(),
            Delegation::Internal,
        )
    };
    node_builder.add_route(Box::new(login_route));

    tree_builder.add_child(node_builder);

    let tree = tree_builder.finalize();
    let response_finalizer = ResponseFinalizerBuilder::new().finalize();

    Router::new(tree, response_finalizer)
}
