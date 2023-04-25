#![allow(unused_imports)]
use ads_client::client::Client;
use ads_proto::proto::ams_address::{AmsAddress, AmsNetId};
use std::net::Ipv4Addr;

fn main() {
    //Create client. If route = None then targed is local machine
    let ams_address = AmsAddress::new(AmsNetId::new(192, 168, 0, 150, 1, 1), 851);
    //let ipv4 = Ipv4Addr::new(192, 168, 0, 150);
    //let mut client = Client::new(ams_address, Some(ipv4));
    let mut client = Client::new(ams_address, None);
    //Connect client
    client.connect().expect("Failed to connect!");

    //Read device info
    let response = client.read_device_info().unwrap();
    println!("Raw response:\n{:?}\n", response);
    println!("Command ID            : {:?}", response.command_id);
    println!("Major version         : {:?}", response.major_version);
    println!("Minor version         : {:?}", response.minor_version);
    println!("Version build         : {:?}", response.version_build);
    println!("ADS Result            : {:?}", response.result);
    println!("Device name bytes     : {:?}", response.device_name);
    println!(
        "Device name String    : {:?}",
        String::from_utf8(response.device_name.to_vec())
    );
}
