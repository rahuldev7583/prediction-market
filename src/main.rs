mod matching_engine;
mod ws;
use crate::{
    matching_engine::{
        EngineCommand, Order,
        Side::{self},
        run_matching_engine,
    },
    ws::{WsState, ws_handler},
};
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use serde::Deserialize;
use tokio::sync::{broadcast, mpsc};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/orderbook")]
async fn get_orderbook(engine_tx: web::Data<mpsc::Sender<EngineCommand>>) -> impl Responder {
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();

    engine_tx
        .send(EngineCommand::GetSnapshot(resp_tx))
        .await
        .unwrap();

    let snapshot = resp_rx.await.unwrap();

    HttpResponse::Ok().json(snapshot)
}

#[derive(Deserialize)]
pub struct OrderRequest {
    pub id: u64,
    pub side: String,
    pub price: u64,
    pub qty: u64,
}

impl From<OrderRequest> for Order {
    fn from(req: OrderRequest) -> Self {
        let side = match req.side.to_lowercase().as_str() {
            "buy" => Side::Buy,
            "sell" => Side::Sell,
            _ => panic!("invalid side"),
        };

        Order {
            id: req.id,
            side,
            price: req.price,
            qty: req.qty,
        }
    }
}

#[post("/orders")]
async fn post_order(
    order_req: web::Json<OrderRequest>,
    engine_tx: web::Data<mpsc::Sender<EngineCommand>>,
) -> impl Responder {
    let order: Order = order_req.into_inner().into();

    if let Err(_) = engine_tx.send(EngineCommand::NewOrder(order)).await {
        return HttpResponse::InternalServerError().body("Engine unavailable");
    }

    HttpResponse::Ok().body("Order sent to engine")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (fill_tx, _) = broadcast::channel(100);
    let (engine_tx, engine_rx) = mpsc::channel(100);

    tokio::spawn(run_matching_engine(engine_rx, fill_tx.clone()));

    let ws_state = WsState { fill_tx };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(ws_state.clone()))
            .app_data(web::Data::new(engine_tx.clone()))
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
