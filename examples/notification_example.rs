#![allow(unused_imports)]
use rust_ads_client::client::Client;
use ads_proto::proto::{
    ads_transition_mode::AdsTransMode,
    ams_address::{AmsAddress, AmsNetId},
};
use std::net::Ipv4Addr;

fn main() {
    //Create client. If route = None then targed is local machine
    let ams_address = AmsAddress::new(AmsNetId::new(192, 168, 0, 150, 1, 1), 851);
    //let ipv4 = Ipv4Addr::new(192, 168, 0, 150);
    //let mut client = Client::new(ams_address, Some(ipv4));
    let mut client = Client::new(ams_address, None);

    //Connect client
    client.connect().expect("Failed to connect!");

    //var name and length
    let var = "Main.counter";
    let len = 2;

    //Subscribe to get notifications when "Main.counter" changes
    let rx = client
        .add_device_notification(var, len, AdsTransMode::OnChange, 1, 1)
        .unwrap();

    //Poll the mpsc receiver for new values
    println!("Receive data...\n");
    let mut list = Vec::new();
    for _ in 1..10 {
        let result = rx.recv();
        if let Ok(r) = result.unwrap() {
            list.push(r.ads_stamp_headers);
        }        
    }

    println!("Print revceived data:\n");
    for r in list {
        println!("{:?}", r);
    }

    //Unsubscribe notifications
    println!("\nDelete the notification");
    let response = client.delete_device_notification(var);
    println!("{:?}", response);
}
