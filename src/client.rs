use ads_proto::ads_services::system_services::*;
use ads_proto::error::AdsError;
use ads_proto::proto::ams_address::{AmsAddress, AmsNetId};
use ads_proto::proto::ams_header::AmsHeader;
use ads_proto::proto::proto_traits::*;
use ads_proto::proto::request::Request;
use ads_proto::proto::response::Response;
use ads_proto::proto::state_flags::StateFlags;
use ads_proto::proto::sumup::sumup_request::SumupReadRequest;
use ads_proto::proto::sumup::sumup_response::SumupReadResponse;
use anyhow;
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

/// UDP ADS-Protocol port dicovery
pub const ADS_UDP_SERVER_PORT: u16 = 48899;
/// TCP ADS-Protocol port not secured
pub const ADS_TCP_SERVER_PORT: u16 = 48898;
/// ADS-Protocol port secured
pub const ADS_SECURE_TCP_SERVER_PORT: u16 = 8016;
//Tcp Header size without response data
pub const AMS_HEADER_SIZE: usize = 38;

pub type ClientResult<T> = Result<T, anyhow::Error>;

#[derive(Debug)]
pub struct Client {
    route: Ipv4Addr,
    ams_targed_address: AmsAddress,
    ams_source_address: AmsAddress,
    stream: Option<TcpStream>,
    ams_header: Option<AmsHeader>,
    invoke_id: u32,
}

impl Client {
    pub fn new(ams_targed_address: AmsAddress, route: Ipv4Addr) -> Self {
        Client {
            route,
            ams_targed_address,
            ams_source_address: AmsAddress::new(AmsNetId::from([0, 0, 0, 0, 0, 0]), 0),
            stream: None,
            ams_header: None,
            invoke_id: 0,
        }
    }

    pub fn connect(&mut self) -> ClientResult<()> {
        if self.stream.is_some() {
            return Ok(());
        }

        let stream = TcpStream::connect(SocketAddr::from((self.route, ADS_TCP_SERVER_PORT)))?;
        stream.set_nodelay(true)?;
        stream.set_write_timeout(Some(Duration::from_millis(1000)))?;
        self.ams_source_address
            .update_from_socket_addr(stream.local_addr()?.to_string().as_str())?; //ToDo update when ads-proto 0.1.1
        self.stream = Some(stream);
        Ok(())
    }

    pub fn request(&self, request: Request) -> ClientResult<Response> {
        let ams_header = self.update_ams_header(request, StateFlags::req_default());
        let stream = self.get_stream()?;
        stream.write_all(&mut self.create_byte_buffer(ams_header));
        Ok(())
    }

    //Private methodes

    fn update_ams_header(&self, request: Request, state_flags: StateFlags) -> AmsHeader {
        match self.ams_header {
            Some(h) => {
                h.update_data(request, StateFlags::req_default());
                h
            }
            None => {
                self.ams_header = Some(AmsHeader::new(
                    self.ams_targed_address,
                    self.ams_source_address,
                    StateFlags::req_default(),
                    self.invoke_id,
                    request,
                ));
                self.ams_header.expect("self.ams_header is None!")
            }
        }
    }

    fn get_stream(&self) -> ClientResult<TcpStream> {
        match self.stream {
            Some(s) => Ok(s),
            None => {
                self.connect()?;
                Ok(self.stream.expect("stream is none!"))
            }
        }
    }

    fn create_byte_buffer(&self, ams_header: AmsHeader) -> Vec<u8> {
        let mut buffer = Vec::new();
        ams_header.write_to(&mut buffer);
        buffer
    }
}
