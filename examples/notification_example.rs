use ads_client::client::Client;
use ads_proto::proto::{
    ads_transition_mode::AdsTransMode,
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

    //var name and length
    let var = "Main.counter";
    let len = 2;

    let rx = client
        .add_device_notification(var, len, AdsTransMode::OnChange, 1, 1)
        .unwrap();

    println!("Receive data...\n");
    let mut list = Vec::new();
    for _ in 1..10 {
        let result = rx.recv().unwrap().unwrap();
        list.push(result.ads_stamp_headers);
    }

    println!("Print revceived data:\n");
    for r in list {
        println!("{:?}", r);
    }

    println!("\nNow delete the notification");
    let response = client.delete_device_notification(var);
    println!("{:?}", response);
}