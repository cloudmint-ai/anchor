use super::{Handler, Router, get, post};
#[cfg(feature = "cloud")]
use super::{middleware, middlewares::context};
#[cfg(feature = "cloud")]
use crate::Database;
use tower_http::compression::CompressionLayer;

pub struct RouterBuilder<E> {
    router: Router<E>,
}

impl<E> RouterBuilder<E>
where
    E: 'static + Clone + Send + Sync,
{
    pub fn new() -> Self {
        Self {
            router: Router::new(),
        }
    }

    pub fn get<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler<T, E>,
        T: 'static,
    {
        self.router = self.router.route(path, get(handler));
        self
    }

    pub fn post<H, T>(mut self, path: &str, handler: H) -> Self
    where
        H: Handler<T, E>,
        T: 'static,
    {
        self.router = self.router.route(path, post(handler));
        self
    }
}

// TODO recover it with account public key supported
// TODO 可以考虑 实现一个 pubkey 的 store 的 trait
// impl<S> RouterBuilder<S>
// where
//     S: VerifiableService + 'static,
//     Arc<S>: Clone + Send + Sync,
// {
//     // with_state 之后，类型变为Router，后续再跟非框架的 layer
//     pub fn with_service(self, service: Arc<S>) -> Router {
//         // 注意和 root<S> 对照修改，注意 layer 次序从前往后依次从外层到内层
//         self.router
//             .with_state(service.clone())
//             .layer(CompressionLayer::new())
//             .layer(middleware::from_fn(trace))
//             .layer(middleware::from_fn_with_state(service, authorize::<S>))
//     }
// }

#[cfg(feature = "cloud")]
impl<E> RouterBuilder<E>
where
    E: 'static + Clone + Send + Sync,
{
    // 所有layer 都需要由 Router 来配置
    // RouterBuilder 保证了先进行 router 配置再进行 layer 配置
    // TODO rename or add with_database
    pub fn with_engine_and_database(self, engine: E, database: Database) -> Router {
        // 注意和 root<S> 对照修改，注意 layer 次序从前往后依次从外层到内层
        self.router
            .with_state(engine.clone())
            .layer(CompressionLayer::new())
            .layer(middleware::from_fn_with_state(Some(database), context))
    }
}

pub fn router<E>() -> RouterBuilder<E>
where
    E: 'static + Clone + Send + Sync,
{
    RouterBuilder::new()
}

pub fn root() -> Router {
    Router::new()
        .route("/health", get(|| async { () }))
        .layer(CompressionLayer::new())
}

// TODO recover it with account supported verifier
// pub fn root<S>(service: Arc<S>) -> Router
// where
//     S: VerifiableService + 'static,
//     Arc<S>: Clone + Send + Sync,
// {
//     Router::new()
//         .route("/health", get(|| async { () }))
//         .route(
//             "/health",
//             post(|| async { () }).layer(middleware::from_fn_with_state(service, authorize::<S>)),
//         )
//         .layer(CompressionLayer::new())
// }

// pub fn root_with_static<A>(application: Arc<A>) -> Router
// where
//     A: VerifiableService + 'static,
//     Arc<A>: Clone + Send + Sync,
// {
//     root(application).nest_service("/static", get_service(ServeDir::new("static")))
// }
