use ads_client::client::Client;
use ads_proto::proto::ams_address::{AmsAddress, AmsNetId};
use ads_proto::proto::request::{Request, ReadDeviceInfoRequest};
use ads_proto::proto::request::ReadRequest;
use ads_proto::ads_services::system_services::*;
use std::net::Ipv4Addr;

fn main() {
    let ams_address = AmsAddress::new(AmsNetId::new(192,168,0,150,1,1), 851);
    let ipv4 = Ipv4Addr::new(192,168,0,150);
    let mut client = Client::new(ams_address, ipv4);

    client.connect().expect("Failed to connect!");
    
    let var = "Main.mi_counter";
    let request = Request::ReadDeviceInfo(ReadDeviceInfoRequest::new());
    let result = client.request(request);
    println!("{:?}", result);
}