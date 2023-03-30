#![allow(unused_imports)]
use ads_client::client::Client;
use ads_proto::proto::ams_address::{AmsAddress, AmsNetId};
use std::net::Ipv4Addr;

fn main() {
    //Create client
    let ams_address = AmsAddress::new(AmsNetId::new(192, 168, 0, 150, 1, 1), 851);
    //let ipv4 = Ipv4Addr::new(192, 168, 0, 150);
    //let mut client = Client::new(ams_address, Some(ipv4));
    let mut client = Client::new(ams_address, None);
    //Connect client
    client.connect().expect("Failed to connect!");

    //Read state
    let response = client.read_state().unwrap();
    println!("Raw state response: {:?}\n", response);
    println!("ADS state     : {:?}", response.ads_state);
    println!("Device state  : {:?}", response.device_state);
    println!("ADS result    : {:?}", response.result);
    println!("Command ID    : {:?}", response.command_id);
}
