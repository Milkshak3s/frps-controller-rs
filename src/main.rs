pub use controller::{self, EndpointState};
use actix_web::{get, middleware, web::Data, App, HttpRequest, HttpResponse, HttpServer, Responder};
use controller::{endpoints, frpclients, FrpClientState};

#[get("/")]
async fn index(_c: Data<EndpointState>, _req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endpt_state = EndpointState::default();
    let endpts_controller = endpoints::run_endpoints_controller(endpt_state.clone());
    let frpc_state = FrpClientState::default();
    let frpc_controller = frpclients::run_frpclient_controller(frpc_state.clone());

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(endpt_state.clone()))
            .app_data(Data::new(frpc_state.clone()))
            .wrap(middleware::Logger::default().exclude("/health"))
            .service(index)
    })
        .bind("0.0.0.0:8080")?
        .shutdown_timeout(5);

    tokio::spawn(endpts_controller);
    tokio::spawn(frpc_controller);
    let _ = tokio::spawn(server.run()).await;
    Ok(())
}
