#![allow(unused, clippy::self_named_constructors)]

use std::cell::{LazyCell, RefCell};
use std::io::{ErrorKind, Read, Write};
use std::mem::replace;
use std::net::{Ipv4Addr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};
use uiua::Value;

pub const DEFAULT_PORT: u16 = 45459;

mod value;

#[cfg(feature = "server")]
pub enum Server {
    Dead,
    Alive {
        join_handle: JoinHandle<io::Error>,
        rx: mpsc::Receiver<Value>,
    },
}

#[cfg(feature = "server")]
impl Server {
    pub fn new() -> Self {
        Self::new_local(DEFAULT_PORT).unwrap()
    }

    pub fn new_local(port: u16) -> io::Result<Self> {
        Self::new_from_addr((Ipv4Addr::LOCALHOST, port))
    }

    pub fn new_from_addr<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let (tx, rx) = mpsc::channel();
        let join_handle = thread::spawn(move || {
            macro_rules! bail {
                ($e: expr) => {
                    match $e {
                        Ok(e) => e,
                        Err(e) => return e,
                    }
                };
            }

            loop {
                let (mut conn, _) = bail!(listener.accept());
                conn.set_read_timeout(Some(Duration::from_millis(1000)))
                    // according to documentation, only crashes if duration is zero.
                    .unwrap_or_else(|_| unreachable!());

                let mut value = Vec::new();
                bail!(conn.read_to_end(&mut value));

                let value = bincode::deserialize(&value).unwrap();
                bail!(tx.send(value).map_err(io::Error::other));
            }
        });
        Ok(Self::Alive { join_handle, rx })
    }

    fn check_thread_died(&mut self) -> io::Result<()> {
        if let Server::Alive { join_handle, .. } = self {
            if join_handle.is_finished() {
                let old_self = replace(self, Server::Dead);
                let Server::Alive { join_handle, .. } = old_self else {
                    unreachable!()
                };
                return Err(join_handle.join().unwrap_or_else(|_| unreachable!()));
            }
        }
        Ok(())
    }

    pub fn try_accept_value(&mut self) -> io::Result<Value> {
        self.check_thread_died()?;
        match self {
            Server::Dead => Err(ErrorKind::NotConnected.into()),
            Server::Alive { join_handle, rx } => rx.try_recv().map_err(|err| match err {
                TryRecvError::Empty => ErrorKind::WouldBlock.into(),
                // this is required since the thread can die between checking and receiving.
                TryRecvError::Disconnected => ErrorKind::WouldBlock.into(),
            }),
        }
    }

    /// Blocking
    pub fn accept_value(&mut self) -> io::Result<Value> {
        self.check_thread_died()?;
        match self {
            Server::Dead => Err(ErrorKind::NotConnected.into()),
            Server::Alive { join_handle, rx } => {
                rx.recv().map_err(|_| ErrorKind::WouldBlock.into())
            }
        }
    }
}

#[cfg(feature = "client")]
struct Connection {
    conn: TcpStream,
}

#[cfg(feature = "client")]
impl Connection {
    pub fn new() -> Self {
        Self::new_local(DEFAULT_PORT).unwrap()
    }

    pub fn new_local(port: u16) -> io::Result<Self> {
        Self::new_from_addr((Ipv4Addr::LOCALHOST, port))
    }

    pub fn new_from_addr<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let conn = TcpStream::connect(addr)?;
        Ok(Self { conn })
    }

    pub fn send_value(&mut self, value: &Value) -> io::Result<()> {
        bincode::serialize_into(&mut self.conn, &value).map_err(|err| match *err {
            bincode::ErrorKind::Io(io) => io,
            other => unreachable!("other serialization error: {other}"),
        })
    }
}

thread_local! {
    static CONNECTION: LazyCell<RefCell<Connection>> = LazyCell::new(|| {
        println!("[fast_desmos2_comms] Initiating global connection...");
        let result = RefCell::new(Connection::new());
        println!("[fast_desmos2_comms] Initiated.");
        result
    });
}

#[cfg(feature = "client")]
pub fn send_value(numbers: Value) {
    CONNECTION.with(|connection| connection.borrow_mut().send_value(&numbers));
}
