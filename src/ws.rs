use actix_web::{Error, HttpRequest, HttpResponse, web};
use futures_util::StreamExt;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct WsState {
    pub fill_tx: broadcast::Sender<String>,
}

pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<WsState>,
) -> Result<HttpResponse, Error> {
    let (res, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let mut rx = state.fill_tx.subscribe();

    actix_web::rt::spawn(async move {
        loop {
            tokio::select! {

                Some(Ok(_msg)) = msg_stream.next() => {

                }

                Ok(msg) = rx.recv() => {
                    if session.text(msg).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    Ok(res)
}
