use futures::{future, pin_mut, StreamExt};

use async_std::prelude::*;
use async_std::task;
use async_tungstenite::async_std::connect_async;
use async_tungstenite::tungstenite::protocol::Message;

async fn run() {
    let connect_addr = "ws://192.168.200.131:8888/ws";

    let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded();
    //发送方
    task::spawn(read_stdin(stdin_tx));

    let (ws_stream, _) = connect_async(connect_addr)
        .await
        .expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");
    //分成发送接收流
    let (write, read) = ws_stream.split();
    //将通道的数据全推给发送流
    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    //定义接收消息操作
    let ws_to_stdout = {
        read.for_each(|message| async {
            match message {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => {
                            println!("Received message: {}", text);
                        }
                        Message::Ping( ping) => {
                            println!("Received ping: {:?}", ping);
                        }
                        _=> {}
                    }
                }
                Err(_) => {}
            }
        })
    };
    //固定一下
    pin_mut!(stdin_to_ws, ws_to_stdout);
    //等待其中一个执行成功
    future::select(stdin_to_ws, ws_to_stdout).await;
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
// 读取用户输出
async fn read_stdin(tx: futures::channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = std::io::stdin();
    loop {
        let mut message = String::new();
        stdin.read_line(&mut message).unwrap();
        tx.unbounded_send(Message::Text( message.into())).unwrap();
    }
}

fn main() {
    task::block_on(run())
}