mod matching_engine;
mod ws;
use crate::{
    matching_engine::{Order, Side::Buy, run_matching_engine},
    ws::{WsState, ws_handler},
};
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use tokio::sync::{broadcast, mpsc};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/orderbook")]
async fn get_orderbook() -> impl Responder {
    HttpResponse::Ok().body("get bids and asks")
}

#[post("/orders")]
async fn post_order(req_body: String, order_tx: web::Data<mpsc::Sender<Order>>) -> impl Responder {
    let order = Order {
        id: 1,
        side: Buy,
        price: 100,
        qty: 10,
    };

    order_tx.send(order).await.unwrap();

    HttpResponse::Ok().body("Order sent to engine")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (fill_tx, _) = broadcast::channel(100);
    let (order_tx, order_rx) = mpsc::channel(100);

    tokio::spawn(run_matching_engine(order_rx, fill_tx.clone()));

    let ws_state = WsState { fill_tx };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(ws_state.clone()))
            .app_data(web::Data::new(order_tx.clone()))
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
