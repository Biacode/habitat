// Copyright (c) 2018 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cell::RefCell;
use std::error;
use std::fmt;
use std::io;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use futures::future::{self, Either};
use futures::prelude::*;
use futures::sync::mpsc;
use protobuf;
use tokio::net::TcpListener;
use tokio_core::reactor;
use tokio_io::AsyncRead;

use super::ListenAddr;
use super::codec::*;
use manager::ctl::*;
use protocols;

pub type CtlSender = mpsc::Sender<SrvWireMessage>;
pub type CtlReceiver = mpsc::Receiver<SrvWireMessage>;

/// Sender from the CtlGateway to the Manager to issue control commands.
pub type MgrSender = mpsc::Sender<CtlCommand>;
/// Receiver from the Manager to the CtlGateway to receive control commands.
pub type MgrReceiver = mpsc::Receiver<CtlCommand>;

#[derive(Debug)]
pub enum CtlCommand {
    SvcLoad(CtlSender, SvcLoadOpts),
}

#[derive(Debug)]
pub enum HandlerError {
    Io(io::Error),
    SendError(mpsc::SendError<CtlCommand>),
}

impl error::Error for HandlerError {
    fn description(&self) -> &str {
        match *self {
            HandlerError::Io(ref err) => err.description(),
            HandlerError::SendError(ref err) => err.description(),
        }
    }
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let content = match *self {
            HandlerError::Io(ref err) => format!("{}", err),
            HandlerError::SendError(ref err) => format!("{}", err),
        };
        write!(f, "{}", content)
    }
}

impl From<io::Error> for HandlerError {
    fn from(err: io::Error) -> Self {
        HandlerError::Io(err)
    }
}

impl From<mpsc::SendError<CtlCommand>> for HandlerError {
    fn from(err: mpsc::SendError<CtlCommand>) -> Self {
        HandlerError::SendError(err)
    }
}

struct Client {
    handle: reactor::Handle,
    state: Rc<RefCell<SrvState>>,
}

impl Client {
    pub fn serve(self, socket: SrvStream) -> Box<Future<Item = (), Error = HandlerError>> {
        let mgr_tx = self.state.borrow().mgr_tx.clone();
        Box::new(self.handshake(socket).and_then(|socket| {
            SrvHandler::new(socket, mgr_tx)
        }))
    }

    fn handshake(&self, socket: SrvStream) -> Box<Future<Item = SrvStream, Error = HandlerError>> {
        let auth_key = self.state.borrow().auth_key.to_string();
        let handshake = socket
            .into_future()
            .map_err(|(err, _)| HandlerError::from(err))
            .and_then(move |(m, io)| {
                m.map_or_else(
                    || {
                        Err(HandlerError::from(
                            io::Error::from(io::ErrorKind::UnexpectedEof),
                        ))
                    },
                    move |m| {
                        if m.message_id() != "Handshake" {
                            debug!("No handshake");
                            return Err(HandlerError::from(
                                io::Error::from(io::ErrorKind::ConnectionAborted),
                            ));
                        }
                        match m.parse::<protocols::ctl::Handshake>() {
                            Ok(decoded) => {
                                trace!("Received handshake, {:?}", decoded);
                                if decoded.get_auth_key() == auth_key {
                                    // JW TODO: handle a case when we receive a non-txn msg
                                    Ok((decoded.transaction.unwrap(), io))
                                } else {
                                    Err(HandlerError::from(
                                        io::Error::from(io::ErrorKind::ConnectionRefused),
                                    ))
                                }
                            }
                            Err(err) => {
                                warn!("Handshake error, {:?}", err);
                                Err(HandlerError::from(
                                    io::Error::from(io::ErrorKind::ConnectionAborted),
                                ))
                            }
                        }
                    },
                )
            })
            .and_then(|(txn, io)| {
                send_complete(io, txn, SrvMessage::<protocols::ctl::NetOk>::new())
            });
        let timeout = reactor::Timeout::new(Duration::from_millis(10_000), &self.handle).unwrap();
        Box::new(handshake.select2(timeout).then(|res| match res {
            Ok(Either::A((hs, _to))) => future::ok(hs),
            Ok(Either::B((_to, _hs))) => {
                future::err(HandlerError::from(
                    io::Error::new(io::ErrorKind::TimedOut, "client timed out"),
                ))
            }
            Err(Either::A((err, _))) => future::err(HandlerError::from(err)),
            Err(Either::B((err, _))) => future::err(HandlerError::from(err)),
        }))
    }
}

/// A `Future` that will resolve into a stream of one or more `SrvWireMessage` replies.
#[must_use = "futures do nothing unless polled"]
struct SrvHandler {
    io: SrvStream,
    state: SrvHandlerState,
    mgr_tx: MgrSender,
    rx: CtlReceiver,
    tx: CtlSender,
}

impl SrvHandler {
    fn new(io: SrvStream, mgr_tx: MgrSender) -> Self {
        let (tx, rx) = mpsc::channel(10);
        SrvHandler {
            io: io,
            state: SrvHandlerState::Receiving,
            mgr_tx: mgr_tx,
            rx: rx,
            tx: tx,
        }
    }
}

impl Future for SrvHandler {
    type Item = ();
    type Error = HandlerError;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            match self.state {
                SrvHandlerState::Receiving => {
                    match try_ready!(self.io.poll()) {
                        Some(msg) => {
                            trace!("OnMessage, {}", msg.message_id());
                            match msg.message_id() {
                                "SvcLoad" => {
                                    let m = msg.parse::<protocols::ctl::SvcLoad>().unwrap();
                                    let opts = SvcLoadOpts::new(m);
                                    let cmd = CtlCommand::SvcLoad(self.tx.clone(), opts);
                                    match self.mgr_tx.start_send(cmd) {
                                        Ok(AsyncSink::Ready) => {
                                            self.state = SrvHandlerState::Sending;
                                            continue;
                                        }
                                        Ok(AsyncSink::NotReady(_)) => return Ok(Async::NotReady),
                                        Err(err) => {
                                            warn!("ManagerReceiver err, {:?}", err);
                                            return Err(HandlerError::from(err));
                                        }
                                    }
                                }
                                _ => {
                                    warn!("Unhandled message, {}", msg.message_id());
                                    break;
                                }
                            }
                        }
                        None => break,
                    }
                }
                SrvHandlerState::Sending => {
                    match self.rx.poll() {
                        Ok(Async::Ready(Some(msg))) => {
                            if msg.is_complete() {
                                self.state = SrvHandlerState::Sent;
                            }
                            try_nb!(self.io.start_send(msg));
                            continue;
                        }
                        Ok(Async::Ready(None)) => self.state = SrvHandlerState::Sent,
                        Ok(Async::NotReady) => return Ok(Async::NotReady),
                        Err(()) => break,
                    }
                }
                SrvHandlerState::Sent => {
                    try_ready!(self.io.poll_complete());
                    trace!("OnMessage complete");
                    break;
                }
            }
        }
        Ok(Async::Ready(()))
    }
}

enum SrvHandlerState {
    /// Handler is Receiving/Waiting for message from client.
    Receiving,
    /// Handler has sent a request to the Manager and is streaming replies back to the client
    /// socket.
    Sending,
    /// All messages have been sent to the client and the Handler is now flushing the connection.
    Sent,
}

struct SrvState {
    auth_key: String,
    mgr_tx: MgrSender,
}

pub fn run(listen_addr: ListenAddr, mgr_tx: MgrSender) {
    thread::Builder::new()
        .name("ctl-gateway".to_string())
        .spawn(move || {
            let mut core = reactor::Core::new().unwrap();
            let handle = core.handle();
            let listener = TcpListener::bind(&listen_addr).unwrap();
            let state = SrvState {
                auth_key: "letmein".to_string(),
                mgr_tx: mgr_tx,
            };
            let state = Rc::new(RefCell::new(state));
            let clients = listener.incoming().map(|socket| {
                let addr = socket.peer_addr().unwrap();
                let io = socket.framed(SrvCodec::new());
                (
                    Client {
                        handle: handle.clone(),
                        state: state.clone(),
                    }.serve(io),
                    addr,
                )
            });
            let server = clients.for_each(|(client, addr)| {
                handle.spawn(client.then(move |res| {
                    debug!("DISCONNECTED from {:?} with result {:?}", addr, res);
                    future::ok(())
                }));
                Ok(())
            });
            core.run(server)
        })
        .expect("ctl-gateway thread start failure");
}

fn _debugf<F: Future<Item = (), Error = ()>>(_: F) {}
fn _debugs<S: Stream<Item = (), Error = ()>>(_: S) {}

fn send_complete<T>(
    socket: SrvStream,
    txn: SrvTxn,
    mut reply: SrvMessage<T>,
) -> Box<Future<Item = SrvStream, Error = HandlerError>>
where
    T: protobuf::Message + protobuf::MessageStatic,
{
    reply.reply_for(txn, true);
    let message = SrvWireMessage::new(reply);
    Box::new(socket.send(message).map_err(|e| HandlerError::from(e)))
}
