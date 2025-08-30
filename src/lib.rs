
use mlua::prelude::{LuaResult, LuaTable};
use mlua::{Function, Lua, UserData, UserDataMethods};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use tungstenite::{connect, Message};

// 为了支持回调，我们需要更复杂的实现
pub struct WebSocketClient {
    /// 用于发送消息的通道
    sender: mpsc::Sender<String>,
    receiver: mpsc::Receiver<String>,
    stop: Arc<Mutex<bool>>,
    socket: Arc<Mutex<tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>>>
}

impl UserData for WebSocketClient {
    /// 添加lua方法
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("send", |_, this, message: String| {
            this.sender.send(message).unwrap();
            Ok(())
        });
        methods.add_method("poll_message",|_, this,callback: Function| {
            match this.receiver.try_recv() {
                Ok(message) => {
                    callback.call::<()>(message)?;
                    Ok(0)
                },
                Err(mpsc::TryRecvError::Empty) => {
                    Ok(1)
                },
                Err(mpsc::TryRecvError::Disconnected) => {
                    eprintln!("Channel disconnected");
                    Ok(-1)
                }
            }
        } );
        methods.add_method("disconnect", |_, this,()| {
            let stop = Arc::clone(&this.stop);
            match stop.lock() {
                Ok(mut stop_guard) => *stop_guard = true,
                Err(_) => eprintln!("Failed to acquire stop lock")
            }

            // 安全地关闭socket
            match this.socket.lock() {
                Ok(mut socket_guard) => {
                    if let ref mut socket = *socket_guard {
                        let _ = socket.close(None);  // 忽略关闭错误
                    }
                }
                Err(_) => eprintln!("Failed to acquire socket lock for disconnect")
            }
            Ok(())
        })

    }
}


///websocket连接
pub fn connect_socket(_: &Lua, url:String) ->LuaResult<WebSocketClient>{
    println!("connecting to {}",url);
    //首先是获取socket连接
    let (ws_stream,_) = connect(&url).expect("Failed to connect");
    // 使用arc来在多个线程间共享socket
    let socket = Arc::new(Mutex::new(ws_stream));
    let receive_socket = Arc::clone(&socket);
    let msg_socket = Arc::clone(&socket);
    // 创建通道
    let (tx,rx) = mpsc::channel::<String>();
    let (tx1,rx1) = mpsc::channel::<String>();
    let stop = Arc::new(Mutex::new(false));
    // 创建一个线程用来接收通道的消息
    let stop_flag = Arc::clone(&stop);
    thread::spawn(move || {
        loop {
            if *stop_flag.lock().unwrap() {
                break;
            }
            let msg = rx.recv();
            match msg {
                Ok(msg) => {
                    // 获取socket
                    let mut socket = receive_socket.lock().unwrap();
                    // 发送消息
                    socket.send(Message::Text(msg.into())).expect("Failed to send message");
                }
                Err(_) => {
                    println!("error msg")
                }
            }
        }
    });
    // 创造一个线程用于处理服务器返回的消息
    let stop_flag1 = Arc::clone(&stop);
    thread::spawn(move || {
        loop {
            if *stop_flag1.lock().unwrap() {
                break;
            }
            let mut socket = msg_socket.lock().unwrap();
            let msg = socket.read().expect("Failed to read message");
            match msg {
                Message::Text(text) => {
                    tx1.send(text.to_string()).expect("Failed to send message");
                }
                Message::Binary(bin) => {
                    // println!("Received binary: {:?}", bin);
                }
                Message::Ping(ping) => {
                    // println!("Received ping: {:?}", ping);
                }
                Message::Pong(pong) => {
                    // println!("Received pong: {:?}", pong);
                }
                _ => {}
            }

        }
    });
    Ok(WebSocketClient {sender: tx,receiver: rx1,stop,socket})
}


#[mlua::lua_module]
fn rust_websocket(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    exports.set("connect", lua.create_function(connect_socket)?)?;
    Ok(exports)
}