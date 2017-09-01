use gotham::router::Router;
use gotham::router::tree::TreeBuilder;
use gotham::router::response::finalizer::ResponseFinalizerBuilder;

/// Builds the subrouter for the Shibboleth-protected part of application, where new sessions will
/// be received for processing.
pub fn receiver() -> Router {
    let tree_builder = TreeBuilder::new();
    let tree = tree_builder.finalize();
    let response_finalizer = ResponseFinalizerBuilder::new().finalize();

    Router::new(tree, response_finalizer)
}
