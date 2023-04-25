#![allow(unused_imports)]
use ads_client::client::Client;
use ads_proto::{
    error::AdsError,
    proto::{
        ams_address::{AmsAddress, AmsNetId},
        response::ReadResponse,
    },
};
use std::{collections::HashMap, net::Ipv4Addr};
fn main() {
    //Create client. If route = None then targed is local machine
    let ams_address = AmsAddress::new(AmsNetId::new(192, 168, 0, 150, 1, 1), 851);
    //let ipv4 = Ipv4Addr::new(192, 168, 0, 150);
    //let mut client = Client::new(ams_address, Some(ipv4));
    let mut client = Client::new(ams_address, None);
    //Connect client
    client.connect().expect("Failed to connect!");

    //var name and length
    let mut var_names = HashMap::new();
    var_names.insert("Main.counter".to_string(), 2);
    var_names.insert("Main.mi_uint".to_string(), 2);
    var_names.insert("Main.mb_bool".to_string(), 1);

    //read data vor all variables in the list with one (tcp) request
    let iterations = 10;
    let mut results: Vec<HashMap<String, ReadResponse>> = Vec::new();
    for _ in 0..iterations {
        match client.sumup_read_by_name(&var_names) {
            Ok(r) => {
                results.push(r);
            }
            Err(e) => {
                if e.is::<AdsError>() {
                    if let Some(e) = e.downcast_ref::<AdsError>() {
                        println!("Some Ads Error: {:?}", e);
                        if client.connect().is_ok() {
                            println!("Reconnected...");
                        } else {
                            println!("Reconnecting failed...");
                        }
                    }
                }
                println!("Other Error: {:?}", e);
            }
        }
    }

    for r in results {
        println!("{:?}", r);
    }
}
