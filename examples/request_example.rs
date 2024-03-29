#![allow(unused_imports)]
use rust_ads_client::client::Client;
use ads_proto::ads_services::system_services::GET_SYMHANDLE_BY_NAME;
use ads_proto::proto::ams_address::{AmsAddress, AmsNetId};
use ads_proto::proto::request::*;
use std::net::Ipv4Addr;

fn main() {
    //Create client. If route = None then targed is local machine
    let ams_address = AmsAddress::new(AmsNetId::new(192, 168, 0, 150, 1, 1), 851);
    //let ipv4 = Ipv4Addr::new(192, 168, 0, 150);
    //let mut client = Client::new(ams_address, Some(ipv4));
    let mut client = Client::new(ams_address, None);
    //Connect client
    client.connect().expect("Failed to connect!");

    //Create the requests manually and supply them to your request
    let mut request_queue = Vec::new();
    request_queue.push(Request::ReadDeviceInfo(ReadDeviceInfoRequest::new()));
    request_queue.push(Request::ReadState(ReadStateRequest::new()));
    let var = "Main.counter";
    request_queue.push(Request::ReadWrite(ReadWriteRequest::new(
        GET_SYMHANDLE_BY_NAME.index_group,
        GET_SYMHANDLE_BY_NAME.index_offset_start,
        4,
        var.as_bytes().to_vec(),
    )));

    //read data directly (wait for response)
    let queue = request_queue.clone();
    for request in queue {
        let result = client.request(request);
        println!("\n{:?}", result);
    }

    //get mpsc tx channel and poll
    let mut rx_queue = Vec::new();
    for request in request_queue {
        rx_queue.push(client.request_rx(request).expect("request_rx failed"));
    }

    let mut counter: u32 = 0;
    let mut wait_count: u32 = 0;
    while counter < 3 {
        for rx in &rx_queue {
            if let Ok(data) = rx.try_recv() {
                println!("\n{:?}", data);
                counter += 1;
            } else {
                wait_count += 1;
            }
        }
    }

    println!("\n{:?} loops while waiting for data", wait_count);
}
