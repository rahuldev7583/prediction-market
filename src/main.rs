mod matching_engine;
mod ws;
use crate::ws::{WsState, ws_handler};
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use tokio::sync::broadcast;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/orderbook")]
async fn get_orderbook() -> impl Responder {
    HttpResponse::Ok().body("get bids and asks")
}

#[post("/orders")]
async fn post_order(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (fill_tx, _) = broadcast::channel(100);

    let ws_state = WsState { fill_tx };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(ws_state.clone()))
            .service(hello)
            .service(get_orderbook)
            .service(post_order)
            .route("/", web::get().to(HttpResponse::Ok))
            .route("/ws", web::get().to(ws_handler))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
