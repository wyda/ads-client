use crate::reader::run_reader_thread;
use crate::request_factory::{self, *};
use ads_proto::error::AdsError;
use ads_proto::proto::ads_state::AdsState;
use ads_proto::proto::ads_transition_mode::AdsTransMode;
use ads_proto::proto::ams_address::{AmsAddress, AmsNetId};
use ads_proto::proto::ams_header::{AmsHeader, AmsTcpHeader};
use ads_proto::proto::proto_traits::*;
use ads_proto::proto::request::{
    ReadDeviceInfoRequest, ReadRequest, ReadStateRequest, Request, WriteRequest,
};
use ads_proto::proto::response::Response;
use ads_proto::proto::response::*;
use ads_proto::proto::state_flags::StateFlags;
use ads_proto::proto::sumup::sumup_request::{SumupReadRequest, SumupWriteRequest};
use ads_proto::proto::sumup::sumup_response::{SumupReadResponse, SumupWriteResponse};
use anyhow::Error;
use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Write;
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpStream};
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

/// UDP ADS-Protocol port discovery
pub const ADS_UDP_SERVER_PORT: u16 = 48899;
/// TCP ADS-Protocol port not secured
pub const ADS_TCP_SERVER_PORT: u16 = 48898;
/// ADS-Protocol port secured
pub const ADS_SECURE_TCP_SERVER_PORT: u16 = 8016;

pub type ClientResult<T> = Result<T, anyhow::Error>;
type TxGeneral = Sender<(u32, Sender<ClientResult<Response>>)>;
type TxNotification = Sender<(u32, Sender<ClientResult<AdsNotificationStream>>)>;
type TxStreamUpdate = Sender<TcpStream>;

#[derive(Debug)]
pub struct Client {
    route: Option<Ipv4Addr>,
    ams_targed_address: AmsAddress,
    ams_source_address: AmsAddress,
    stream: Option<TcpStream>,
    invoke_id: u32,
    tx_general: Option<TxGeneral>,
    tx_notification: Option<TxNotification>,
    tx_stream_update: Option<TxStreamUpdate>,
    thread_started: bool,
    handle_list: HashMap<String, u32>,
    notification_handle_list: HashMap<String, u32>,
}

impl Drop for Client {
    fn drop(&mut self) {
        if let Some(s) = &self.stream {
            let _ = s.shutdown(Shutdown::Both);
        }
    }
}

impl Client {
    /// Setup a new client. This will will not yet connect to the targed.
    /// Call connect() after creation.
    pub fn new(ams_targed_address: AmsAddress, route: Option<Ipv4Addr>) -> Self {
        Client {
            route,
            ams_targed_address,
            ams_source_address: AmsAddress::new(AmsNetId::from([0, 0, 0, 0, 0, 0]), 0),
            stream: None,
            invoke_id: 0,
            tx_general: None,
            tx_notification: None,
            tx_stream_update: None,
            thread_started: false,
            handle_list: HashMap::new(),
            notification_handle_list: HashMap::new(),
        }
    }

    /// Connect to host and start reader thread.
    /// Fails if host is not reachable or if the reader thread can't be started.
    pub fn connect(&mut self) -> ClientResult<ReadStateResponse> {
        if self.stream.is_none() {
            self.stream = Some(self.create_stream()?);
            if self.route.is_none() {
                self.open_local_port()?;
            }
        }

        if let Some(stream) = &self.stream {
            if self.route.is_some() {
                self.ams_source_address
                    .update_from_socket_addr(stream.local_addr()?)?;
            }

            if !self.thread_started {
                let (tx, rx) = channel::<(u32, Sender<ClientResult<Response>>)>();
                let (tx_not, rx_not) =
                    channel::<(u32, Sender<ClientResult<AdsNotificationStream>>)>();
                let (tx_tcp, rx_tcp) = channel::<TcpStream>();
                self.tx_general = Some(tx);
                self.tx_notification = Some(tx_not);
                self.tx_stream_update = Some(tx_tcp);
                self.thread_started = run_reader_thread(stream.try_clone()?, rx, rx_not, rx_tcp)?;
            } else if let Some(tx) = &self.tx_stream_update {
                tx.send(stream.try_clone()?)?;
            }
            //Check if host is responding
            self.read_state()
        } else {
            Err(anyhow!(AdsError::ErrPortNotConnected))
        }
    }

    /// Create the TCP stream
    fn create_stream(&mut self) -> ClientResult<TcpStream> {
        let mut route = Ipv4Addr::from_str("127.0.0.1")?;
        if let Some(r) = self.route {
            route = r;
        }

        let stream = TcpStream::connect(SocketAddr::from((route, ADS_TCP_SERVER_PORT)))?;
        stream.set_nodelay(true)?;
        stream.set_write_timeout(Some(Duration::from_millis(1000)))?;
        stream.set_read_timeout(Some(Duration::from_millis(1000)))?;
        Ok(stream)
    }

    /// open local port in case of local machine
    fn open_local_port(&mut self) -> ClientResult<()> {
        let request_port_msg = [0, 16, 2, 0, 0, 0, 0, 0];
        let mut buf = [0; 14];

        if let Some(s) = &mut self.stream {
            s.write_all(&request_port_msg).unwrap();
            use std::io::Read;
            s.read_exact(&mut buf)?;
            let (_, mut buf_split) = buf.split_at(6);
            let ams_address = AmsAddress::read_from(&mut buf_split);
            self.ams_source_address = ams_address.unwrap();
        }
        Ok(())
    }

    /// Sends a request and returns a Result<Response>
    /// Blocks until the response has been received or on error occures
    /// Fails if no tcp stream is available.
    pub fn request(&mut self, request: Request) -> ClientResult<Response> {
        let rx = self.request_rx(request)?;
        let response = rx.recv()?;
        self.check_tcp_stream(&response);
        response
    }

    /// Sends a request and returns imediatly a receiver object to read from (mpsc::Receiver).
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
        Err(anyhow!(AdsError::AdsErrClientPortNotOpen))
    }

    /// Read a var value by it's name.
    /// Returns ReadResponse
    pub fn read_by_name(&mut self, var_name: &str, len: u32) -> ClientResult<ReadResponse> {
        let handle = self.get_var_handle(var_name)?;
        let request = Request::Read(request_factory::get_read_request(handle, len));
        let response = self.request(request)?;
        let read_response: ReadResponse = response.try_into()?;
        Ok(read_response)
    }

    /// Read a list of var values by name. This will bundle all requested variables into a single request.
    /// Returns a HashMap<String, ReadResponse>
    pub fn sumup_read_by_name(
        &mut self,
        var_list: &HashMap<String, u32>,
    ) -> ClientResult<HashMap<String, ReadResponse>> {
        let mut requests: Vec<ReadRequest> = Vec::new();
        let mut var_names: Vec<String> = Vec::new();
        for name in var_list.keys() {
            var_names.push(name.clone());
        }

        let handles = self.sumup_get_var_handle(&var_names)?;
        for (var, length) in var_list {
            if let Some(h) = handles.get(var) {
                requests.push(get_read_request(*h, *length));
            }
        }

        let mut buf = Vec::new();
        let sumup_request = SumupReadRequest::new(requests);
        sumup_request.write_to(&mut buf)?;
        let request = Request::ReadWrite(get_sumup_read_request(
            sumup_request.request_count(),
            sumup_request.expected_response_len(),
            buf,
        ));
        let response = self.request(request)?;
        let read_write_response: ReadWriteResponse = response.try_into()?;
        let sumup_read_response =
            SumupReadResponse::read_from(&mut read_write_response.data.as_slice())?;
        let mut result: HashMap<String, ReadResponse> = HashMap::new();

        if read_write_response.result == AdsError::ErrNoError {
            for (n, (name, _)) in var_list.iter().enumerate() {
                result.insert(name.clone(), sumup_read_response.read_responses[n].clone());
            }
        } else {
            return Err(anyhow![read_write_response.result]);
        }
        Ok(result)
    }

    /// Write by name
    /// Returns WriteResponse
    pub fn write_by_name(&mut self, var_name: &str, data: Vec<u8>) -> ClientResult<WriteResponse> {
        let handle = self.get_var_handle(var_name)?;
        let request = Request::Write(request_factory::get_write_request(handle, data));
        let response = self.request(request)?;
        let write_response: WriteResponse = response.try_into()?;
        Ok(write_response)
    }

    /// Write a list of var values by name. This will bundle all the write data into a single write request.
    /// Returns a HashMap<String, WriteResponse>
    pub fn sumup_write_by_name(
        &mut self,
        var_list: HashMap<String, Vec<u8>>,
    ) -> ClientResult<HashMap<String, WriteResponse>> {
        let mut requests: Vec<WriteRequest> = Vec::new();
        for (varname, data) in &var_list {
            let handle = self.get_var_handle(varname.as_str())?;
            requests.push(get_write_request(handle, data.clone()));
        }

        let mut buf = Vec::new();
        let sumup_request = SumupWriteRequest::new(requests);
        sumup_request.write_to(&mut buf)?;
        let request = Request::ReadWrite(get_sumup_write_request(
            sumup_request.request_count(),
            sumup_request.expected_response_len(),
            buf,
        ));
        let response = self.request(request)?;
        let read_write_response: ReadWriteResponse = response.try_into()?;
        let sumup_write_response =
            SumupWriteResponse::read_from(&mut read_write_response.data.as_slice())?;
        let mut result: HashMap<String, WriteResponse> = HashMap::new();

        if read_write_response.result == AdsError::ErrNoError {
            for (n, (name, _)) in var_list.iter().enumerate() {
                result.insert(
                    name.clone(),
                    sumup_write_response.write_responses[n].clone(),
                );
            }
        } else {
            return Err(anyhow![read_write_response.result]);
        }
        Ok(result)
    }

    /// Read device info
    /// Returns ReadDeviceInfoResponse
    pub fn read_device_info(&mut self) -> ClientResult<ReadDeviceInfoResponse> {
        let request = Request::ReadDeviceInfo(ReadDeviceInfoRequest::new());
        let response = self.request(request)?;
        let device_info_response: ReadDeviceInfoResponse = response.try_into()?;
        Ok(device_info_response)
    }

    /// Read PLC state
    /// Returns ReadStateResponse
    pub fn read_state(&mut self) -> ClientResult<ReadStateResponse> {
        let request = Request::ReadState(ReadStateRequest::new());
        let response = self.request(request)?;
        let device_state: ReadStateResponse = response.try_into()?;
        Ok(device_state)
    }

    /// Write control
    /// Returns WriteControlResponse
    pub fn write_control(
        &mut self,
        ads_state: AdsState,
        device_state: u16,
    ) -> ClientResult<WriteControlResponse> {
        let request = Request::WriteControl(request_factory::get_write_control_request(
            ads_state,
            device_state,
        ));
        let response = self.request(request)?;
        let write_control_response: WriteControlResponse = response.try_into()?;
        Ok(write_control_response)
    }

    /// Read and write data
    /// Returns ReadWriteResponse
    pub fn read_write(
        &mut self,
        index_offset: u32,
        read_len: u32,
        write_data: Vec<u8>,
    ) -> ClientResult<ReadWriteResponse> {
        let request = Request::ReadWrite(request_factory::get_read_write_request(
            index_offset,
            read_len,
            write_data,
        ));
        let response = self.request(request)?;
        let read_write_response: ReadWriteResponse = response.try_into()?;
        Ok(read_write_response)
    }

    /// Add device notification to receive updated values at value change or at a certain time interfall
    /// Returns mpsc::receiver which can be polled
    pub fn add_device_notification(
        &mut self,
        var_name: &str,
        length: u32,
        transmission_mode: AdsTransMode,
        max_delay: u32,
        cycle_time: u32,
    ) -> ClientResult<Receiver<Result<AdsNotificationStream, Error>>> {
        let handle = self.get_var_handle(var_name)?;
        let request = Request::AddDeviceNotification(request_factory::get_add_device_notification(
            handle,
            length,
            transmission_mode,
            max_delay,
            cycle_time,
        ));

        //Get notification handle
        let response: AddDeviceNotificationResponse = self.request(request)?.try_into()?;
        let handle = response.notification_handle;
        //Create mpsc channel for notifications
        let (tx, rx) = channel::<ClientResult<AdsNotificationStream>>();
        //Send tx to reader thread
        self.get_notification_tx()?
            .send((handle, tx))
            .expect("Failed to send request to thread by mpsc channel");

        self.notification_handle_list
            .insert(var_name.to_string(), handle);
        Ok(rx)
    }

    /// Release a device notification on the host
    /// Returns DeleteDeviceNotificationResponse
    pub fn delete_device_notification(
        &mut self,
        var_name: &str,
    ) -> ClientResult<DeleteDeviceNotificationResponse> {
        let handle;
        if let Some(h) = self.notification_handle_list.get(var_name) {
            handle = *h;
            let request = Request::DeleteDeviceNotification(
                request_factory::get_delete_device_notification(handle),
            );
            let response = self.request(request)?;
            let response: DeleteDeviceNotificationResponse = response.try_into()?;
            self.notification_handle_list.remove(var_name);
            return Ok(response);
        }
        Err(anyhow!(AdsError::AdsErrDeviceSymbolNotFound)) //??
    }

    fn get_var_handle(&mut self, var_name: &str) -> ClientResult<u32> {
        if let Some(handle) = self.handle_list.get(var_name) {
            Ok(*handle)
        } else {
            let handle = self.request_var_handle(var_name)?;
            self.handle_list.insert(var_name.to_string(), handle);
            Ok(handle)
        }
    }

    fn sumup_get_var_handle(
        &mut self,
        var_names: &Vec<String>,
    ) -> ClientResult<HashMap<String, u32>> {
        let mut do_request: Vec<String> = Vec::new();
        let mut handles: HashMap<String, u32> = HashMap::new();
        for var in var_names {
            if let Some(handle) = self.handle_list.get(var) {
                handles.insert(var.clone(), *handle);
            } else {
                do_request.push(var.clone());
            }
        }

        if !do_request.is_empty() {
            let requested_handles = self.sumup_request_var_handle(&do_request)?;
            for (name, handle) in requested_handles {
                self.handle_list.insert(name.clone(), handle);
                handles.insert(name, handle);
            }
        }
        Ok(handles)
    }

    /// Request new var handle
    fn request_var_handle(&mut self, var_name: &str) -> ClientResult<u32> {
        let request = Request::ReadWrite(get_var_handle_request(var_name));
        let response: ReadWriteResponse = self.request(request)?.try_into()?;

        if response.length == 4 {
            return Ok(response.data.as_slice().read_u32::<LittleEndian>()?);
        }
        Err(anyhow!(
            "Failed to get var handle! Variable {} not found!",
            var_name
        ))
    }

    /// Sumup a var handle request
    /// Not really a sumup request. This methode send for each handle request a tx.
    // To Do. Is there a way to perform a sumup for handle requests?
    fn sumup_request_var_handle(
        &mut self,
        var_list: &Vec<String>,
    ) -> ClientResult<HashMap<String, u32>> {
        let mut result: HashMap<String, u32> = HashMap::new();
        for var in var_list {
            let response = self.request(Request::ReadWrite(get_var_handle_request(var)))?;
            let handle: ReadWriteResponse = response.try_into()?;
            let handle = handle.data.as_slice().read_u32::<LittleEndian>()?;
            result.insert(var.clone(), handle);
        }
        Ok(result)
    }

    /// Release var handle
    pub fn release_handle(&mut self, var_name: &str) -> ClientResult<WriteResponse> {
        if let Some(handle) = self.handle_list.get(var_name) {
            let request = Request::Write(request_factory::get_release_handle_request(*handle));
            let response = self.request(request)?;
            let response: WriteResponse = response.try_into()?;
            self.handle_list.remove(var_name);
            return Ok(response);
        }
        Err(anyhow!("Handle not available"))
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

    ///Check if stream disconnected
    fn check_tcp_stream(&mut self, response: &ClientResult<Response>) {
        if let Err(e) = response {
            if e.is::<AdsError>() {
                let e = e.downcast_ref::<AdsError>();
                if let Some(e) = e {
                    if e == &AdsError::ErrPortNotConnected {
                        if let Some(stream) = &self.stream {
                            let _ = stream.shutdown(Shutdown::Both);
                        }
                        self.handle_list.clear();
                        self.notification_handle_list.clear();
                        self.stream = None;
                    }
                }
            }
        }
    }

    /// Gets the tx (mpsc::sender) to notify the reader thread about a new handle
    fn get_general_tx(&self) -> ClientResult<&TxGeneral> {
        if let Some(tx) = &self.tx_general {
            return Ok(tx);
        }
        Err(anyhow!(AdsError::AdsErrClientError)) //ToDo create better error
    }

    /// Gets the tx (mpsc::sender) to notify the reader thread about a new notification handle
    fn get_notification_tx(&self) -> ClientResult<&TxNotification> {
        if let Some(tx) = &self.tx_notification {
            return Ok(tx);
        }
        Err(anyhow!(AdsError::AdsErrClientError)) //ToDo create better error
    }
}
