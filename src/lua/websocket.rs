use std::{collections::HashMap, net::SocketAddr, sync::Mutex};

use super::ws::WSMessage;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use hyper::upgrade::Upgraded;
use mlua::prelude::*;
use once_cell::sync::Lazy;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

type Tx = UnboundedSender<Message>;

static SUBSCRIBERS: Lazy<Mutex<HashMap<SocketAddr, Tx>>> = Lazy::new(Default::default);

#[derive(Clone)]
pub struct WSSender {
    sender: Tx,
    addr: SocketAddr,
}

impl LuaUserData for WSSender {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_method_mut("send", |_, this, msg: LuaAnyUserData| {
            let msg = msg.take::<WSMessage>()?;
            this.sender.unbounded_send(msg.0).to_lua_err()?;
            Ok(())
        });
        _methods.add_method_mut("disconnect", |_, this, ()| {
            let map = SUBSCRIBERS.lock();
            if let Ok(mut m) = map {
                m.remove(&this.addr);
                this.sender.disconnect();
                this.sender.close_channel();
            }
            Ok(())
        });
        _methods.add_method("is_closed", |_, this, ()| Ok(this.sender.is_closed()));
    }
}

#[derive(Clone)]
pub struct WSPeerMap;

impl LuaUserData for WSPeerMap {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_function("send_all", |_, msg: LuaAnyUserData| {
            let msg = msg.take::<WSMessage>()?;
            let map = SUBSCRIBERS.lock();
            if let Ok(m) = map {
                for (_, sender) in m.iter() {
                    sender.unbounded_send(msg.0.clone()).to_lua_err()?;
                }
            }
            Ok(())
        });
        _methods.add_function(
            "send",
            |_, (addr, msg): (LuaAnyUserData, LuaAnyUserData)| {
                let msg = msg.take::<WSMessage>()?;
                let addr = addr.borrow::<WSAddr>()?;
                SUBSCRIBERS
                    .lock()
                    .unwrap()
                    .get(&addr.0)
                    .unwrap()
                    .unbounded_send(msg.0)
                    .to_lua_err()?;
                Ok(())
            },
        );
        _methods.add_function("remove", |_, addr: LuaAnyUserData| {
            let addr = addr.borrow::<WSAddr>()?;
            let mut sender = SUBSCRIBERS.lock().unwrap().remove(&addr.0).unwrap();
            sender.disconnect();
            sender.close_channel();
            Ok(())
        });
    }
}

pub struct WSAddr(SocketAddr);

impl LuaUserData for WSAddr {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(_methods: &mut M) {
        _methods.add_method("send", |_, this, msg: LuaAnyUserData| {
            let msg = msg.take::<WSMessage>()?;
            SUBSCRIBERS
                .lock()
                .unwrap()
                .get(&this.0)
                .unwrap()
                .unbounded_send(msg.0)
                .to_lua_err()?;
            Ok(())
        });
        _methods.add_method("disconnect", |_, this, ()| {
            let mut sender = SUBSCRIBERS.lock().unwrap().remove(&this.0).unwrap();
            sender.disconnect();
            sender.close_channel();
            Ok(())
        });
    }
}

pub async fn handle_connection(
    handler: LuaFunction<'static>,
    ws_stream: WebSocketStream<Upgraded>,
    addr: SocketAddr,
) -> LuaResult<()> {
    println!("WebSocket connection established: {addr}");

    let (tx, rx) = unbounded::<Message>();

    SUBSCRIBERS.lock().unwrap().insert(addr, tx.clone());

    let (outgoing, incoming) = ws_stream.split();

    // let sender = WSSender { sender: tx, addr };

    let broadcast_incoming = incoming.try_for_each(|msg| {
        handler
            .call::<_, ()>((WSPeerMap, WSAddr(addr), WSMessage(msg)))
            .unwrap();
        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    Ok(())
}
