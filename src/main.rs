pub use controller::{self, State};
use actix_web::{get, middleware, web::Data, App, HttpRequest, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn index(_c: Data<State>, _req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = State::default();
    let controller = controller::run(state.clone());

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .wrap(middleware::Logger::default().exclude("/health"))
            .service(index)
    })
        .bind("0.0.0.0:8080")?
        .shutdown_timeout(5);

    tokio::join!(controller, server.run()).1?;
    Ok(())
}
