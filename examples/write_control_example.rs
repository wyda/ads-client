use ads_client::client::Client;
use ads_proto::proto::{
    ads_state::AdsState,
    ams_address::{AmsAddress, AmsNetId},
};
use std::net::Ipv4Addr;

fn main() {
    //Create client
    let ams_address = AmsAddress::new(AmsNetId::new(192, 168, 0, 150, 1, 1), 851);
    let ipv4 = Ipv4Addr::new(192, 168, 0, 150);
    let mut client = Client::new(ams_address, ipv4);
    //Connect client
    client.connect().expect("Failed to connect!");

    //Set PLC to stop
    let ads_state = AdsState::AdsStateStop;
    let response = client.write_control(ads_state, 0).unwrap();
    println!("Command id    :{:?}", response.command_id);
    println!("ADS result    :{:?}", response.result);
}
