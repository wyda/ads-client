use crate::reader::run_reader_thread;
use ads_proto::error::AdsError;
use ads_proto::proto::ams_address::{AmsAddress, AmsNetId};
use ads_proto::proto::ams_header::{AmsHeader, AmsTcpHeader};
use ads_proto::proto::proto_traits::*;
use ads_proto::proto::request::Request;
use ads_proto::proto::response::Response;
use ads_proto::proto::response::*;
use ads_proto::proto::state_flags::StateFlags;
use anyhow::{anyhow, Result};
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

/// UDP ADS-Protocol port dicovery
pub const ADS_UDP_SERVER_PORT: u16 = 48899;
/// TCP ADS-Protocol port not secured
pub const ADS_TCP_SERVER_PORT: u16 = 48898;
/// ADS-Protocol port secured
pub const ADS_SECURE_TCP_SERVER_PORT: u16 = 8016;

pub type ClientResult<T> = Result<T, anyhow::Error>;
type TxGeneral = Sender<(u32, Sender<ClientResult<Response>>)>;
type TxNotification = Sender<(u32, Sender<ClientResult<AdsNotificationStream>>)>;

#[derive(Debug)]
pub struct Client {
    route: Ipv4Addr,
    ams_targed_address: AmsAddress,
    ams_source_address: AmsAddress,
    stream: Option<TcpStream>,
    invoke_id: u32,
    tx_general: Option<TxGeneral>,
    tx_notification: Option<TxNotification>,
    thread_started: bool,
}

impl Client {
    pub fn new(ams_targed_address: AmsAddress, route: Ipv4Addr) -> Self {
        Client {
            route,
            ams_targed_address,
            ams_source_address: AmsAddress::new(AmsNetId::from([0, 0, 0, 0, 0, 0]), 0),
            stream: None,
            invoke_id: 0,
            tx_general: None,
            tx_notification: None,
            thread_started: false,
        }
    }

    pub fn connect(&mut self) -> ClientResult<()> {
        if self.stream.is_none() {
            self.stream = Some(self.create_stream()?);
        }

        if let Some(stream) = &self.stream {
            self.ams_source_address
                .update_from_socket_addr(stream.local_addr()?)?;

            if !self.thread_started {
                let (tx, rx) = channel::<(u32, Sender<ClientResult<Response>>)>();
                let (tx_not, rx_not) =
                    channel::<(u32, Sender<ClientResult<AdsNotificationStream>>)>();
                self.tx_general = Some(tx);
                self.tx_notification = Some(tx_not);
                self.thread_started = run_reader_thread(stream.try_clone()?, rx, rx_not)?;
            }
        }
        Ok(())
    }

    fn create_stream(&self) -> ClientResult<TcpStream> {
        let stream = TcpStream::connect(SocketAddr::from((self.route, ADS_TCP_SERVER_PORT)))?;
        stream.set_nodelay(true)?;
        stream.set_write_timeout(Some(Duration::from_millis(1000)))?;
        Ok(stream)
    }

    /// Sends a reqest to the remote device and returns a Result<Response>
    /// Blocks until the response has been received or on error occured
    /// Fails if no tcp stream is available.
    pub fn request(&mut self, request: Request) -> ClientResult<Response> {
        let rx = self.request_rx(request)?;
        rx.recv()?
    }

    /// Sends a request to the remote device
    /// and returns imediatly a receiver object to read from (mpsc::Receiver).
    /// Fails if no tcp stream is available.
    pub fn request_rx(&mut self, request: Request) -> ClientResult<Receiver<Result<Response>>> {
        let ams_header = self.new_tcp_ams_request_header(request);
        let (tx, rx) = channel::<ClientResult<Response>>();
        self.get_general_tx()?
            .send((self.invoke_id, tx))
            .expect("Failed to send request to thread by mpsc channel");
        let mut buffer = Vec::new();

        ams_header.write_to(&mut buffer)?;

        if let Some(s) = &mut self.stream {
            s.write_all(&buffer)?;
            return Ok(rx);
        }
        Err(anyhow!(AdsError::AdsErrClientPortNotOpen)) //ToDo improve error
    }

    ///Create new tcp_ams_header with supplied request data.
    fn new_tcp_ams_request_header(&mut self, request: Request) -> AmsTcpHeader {
        self.invoke_id += 1;
        AmsTcpHeader::from(AmsHeader::new(
            self.ams_targed_address.clone(),
            self.ams_source_address.clone(),
            StateFlags::req_default(),
            self.invoke_id,
            request,
        ))
    }

    fn get_general_tx(&self) -> ClientResult<&TxGeneral> {
        if let Some(tx) = &self.tx_general {
            return Ok(tx);
        }
        Err(anyhow!(AdsError::AdsErrClientError)) //ToDo create better error
    }
}
