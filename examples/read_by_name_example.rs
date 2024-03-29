#![allow(unused_imports)]
use rust_ads_client::client::Client;
use ads_proto::{
    error::AdsError,
    proto::ams_address::{AmsAddress, AmsNetId},
};
use std::net::Ipv4Addr;

fn main() {
    //Create client. If route = None then targed is local machine
    let ams_address = AmsAddress::new(AmsNetId::new(192, 168, 0, 150, 1, 1), 851);
    //let ipv4 = Ipv4Addr::new(127, 0, 0, 1);
    //let mut client = Client::new(ams_address, Some(ipv4));
    let mut client = Client::new(ams_address, None);
    //Connect client
    client.connect().expect("Failed to connect!");

    //var name and length
    let var = "Main.counter";
    let len = 2;

    //read data by name
    let iterations = 10;
    println!("Read var {:?} {:?} times", var, iterations);
    for _ in 0..iterations {
        match client.read_by_name(var, len) {
            Ok(r) => {
                println!("{:?}", r);
            }
            Err(e) => {
                if e.is::<AdsError>() {
                    if let Some(e) = e.downcast_ref::<AdsError>() {
                        println!("Ads Error{:?}", e);
                        if client.connect().is_ok() {
                            println!("Reconnected...");
                        } else {
                            println!("Reconnecting failed...");
                        }
                    }
                }
                println!("Other Error{:?}", e);
            }
        }
    }

    //Release handle if not needed anymore
    let result = client.release_handle(var);
    println!("{:?}", result.unwrap());
}
