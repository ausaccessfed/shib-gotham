extern crate chrono;
extern crate fern;
extern crate hyper;
extern crate log;
extern crate mime;
#[macro_use]
extern crate serde_derive;

extern crate gotham;
extern crate shib_gotham;

use log::LevelFilter;
use hyper::{Response, StatusCode};
use gotham::pipeline::new_pipeline;
use gotham::pipeline::set::*;
use gotham::middleware::session::{NewSessionMiddleware, SessionData};
use gotham::http::response::create_response;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::{FromState, State};
use shib_gotham::{AuthenticatedSession, ReceiverFailed, Shibbleware};

fn main() {
    set_logging();
    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router())
}

fn set_logging() {
    fern::Dispatch::new()
        .level(LevelFilter::Error)
        .level_for("gotham", log::LevelFilter::Trace)
        .level_for("gotham::state", log::LevelFilter::Error)
        .level_for("todo_session", log::LevelFilter::Error)
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

#[derive(Default, Serialize, Deserialize)]
struct Session {
    user: Option<UserAttributes>,
}

impl AuthenticatedSession for Session {
    fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }
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

    pub fn attributes(state: State) -> (State, Response) {
        let body = format!(
            "
                <html>
                    <head>
                        <title>shib-gotham - Attribute Reflector Example</title>
                    </head>
                    <body>
                        <h2>Attributes</h2>
                        <pre><code>{:?}</code></pre>
                    </body>
                </html>
            ",
            SessionData::<Session>::borrow_from(&state)
                .user
                .as_ref()
                .unwrap()
        );

        let response = create_response(
            &state,
            StatusCode::Ok,
            Some((body.into_bytes(), mime::TEXT_HTML)),
        );

        (state, response)
    }
}

fn receive_subject(state: &mut State, attributes: UserAttributes) -> Result<(), ReceiverFailed> {
    println!("received attributes: {:?}", attributes);

    SessionData::<Session>::borrow_mut_from(state).user = Some(attributes);

    Ok(())
}

fn router() -> Router {
    let pipelines = new_pipeline_set();

    let (pipelines, default) = pipelines.add(
        new_pipeline()
            .add(
                NewSessionMiddleware::default()
                    .with_session_type::<Session>()
                    .insecure(),
            )
            .build(),
    );

    let (pipelines, protected) = pipelines.add(
        new_pipeline()
            .add(Shibbleware::<Session>::new("/auth/login"))
            .build(),
    );

    let pipelines = finalize_pipeline_set(pipelines);

    let default_pipeline_chain = (default, ());
    let protected_pipeline_chain = (protected, (default, ()));

    let protected_router = build_router(protected_pipeline_chain, pipelines.clone(), |route| {
        route.get("/attributes").to(controller::attributes);
    });

    build_router(default_pipeline_chain, pipelines, |route| {
        route.get("/").to(controller::welcome);

        route
            .delegate_without_pipelines("/protected")
            .to_router(protected_router);

        route
            .delegate("/auth")
            .to_router(shib_gotham::auth_router(receive_subject));
    })
}
