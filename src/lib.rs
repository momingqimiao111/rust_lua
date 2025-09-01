use futures::{future, pin_mut, StreamExt};


use async_std::task;
use async_tungstenite::async_std::connect_async;
use async_tungstenite::tungstenite::protocol::Message;
use futures::channel::mpsc::{TryRecvError, UnboundedReceiver, UnboundedSender};
use mlua::prelude::{LuaResult, LuaTable};
use mlua::{Function, Lua, UserData, UserDataMethods};

struct WebSocketClient {
    /// 用于发送消息的通道
    sender: UnboundedSender<Message>,
    receiver: UnboundedReceiver<String>,
}

impl UserData for WebSocketClient {
    /// 添加lua方法
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("send", |_, this, message: String| {
            this.sender.unbounded_send(Message::Text(message.into())).unwrap();
            Ok(())
        });
        methods.add_method_mut("poll_message", |_, this, callback: Function| {
            match this.receiver.try_next() {
                Ok(Some(msg)) => {
                    println!("获得一条消息");
                    callback.call::<()>(msg).expect("TODO: panic message");
                },
                Ok(None)=> {},
                Err(_) => {}
            }
            Ok(())
        })
    }
}

async fn connect_socket(url:String,tx:UnboundedSender<String>,rx:UnboundedReceiver<Message>){
    println!("connecting to {}", url);
    //创建socket连接
    let (ws_stream, _) = connect_async(url)
        .await
        .expect("Failed to connect");
    //将socket流拆分成发送和接收
    let (write, read) = ws_stream.split();
    //将通道的消息都发给发送流
    let send_to_ws = rx.map(Ok).forward(write);
    //定义输出的操作
    let ws_to_lua = {
        read.for_each(|message| async {
            match message {
                Ok(msg) =>{
                    match msg {
                        Message::Text(text) => {
                            println!("Received message: {}", text);
                            // callback.call::<()>(text.to_string()).expect("TODO: panic message");
                            println!("接收到消息");
                            //发送到通道
                            tx.unbounded_send(text.to_string()).unwrap();
                        }
                        Message::Ping(_) => {
                            // println!("Received ping: {:?}", ping);
                        }
                        _=> {}
                    }
                },
                Err(_)=>{}
            }
        })
    };
    pin_mut!(send_to_ws, ws_to_lua);
    future::select(send_to_ws, ws_to_lua).await;
}
fn connect(lua: &mlua::Lua, url:String)->LuaResult<WebSocketClient> {
    let (stdin_tx, stdin_rx) = futures::channel::mpsc::unbounded();
    let (stdout_tx, stdout_rx) = futures::channel::mpsc::unbounded();
    task::spawn(connect_socket(url,stdout_tx,stdin_rx));
    Ok(WebSocketClient{
        sender: stdin_tx,
        receiver: stdout_rx
    })
}

#[mlua::lua_module]
fn rust_websocket(lua: &Lua) -> LuaResult<LuaTable> {
    let socket_table = lua.create_table()?;
    socket_table.set("connect", lua.create_function(connect)?)?;
    Ok(socket_table)
}