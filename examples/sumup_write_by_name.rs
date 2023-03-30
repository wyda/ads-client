#![allow(unused_imports)]
use ads_client::client::Client;
use ads_proto::{
    error::AdsError,
    proto::{
        ams_address::{AmsAddress, AmsNetId},
        response::WriteResponse,
    },
};
use std::{collections::HashMap, net::Ipv4Addr};

fn main() {
    //Create client
    let ams_address = AmsAddress::new(AmsNetId::new(192, 168, 0, 150, 1, 1), 851);
    //let ipv4 = Ipv4Addr::new(192, 168, 0, 150);
    //let mut client = Client::new(ams_address, Some(ipv4));
    let mut client = Client::new(ams_address, None);
    //Connect client
    client.connect().expect("Failed to connect!");

    //var name and length
    let mut var_names: HashMap<String, Vec<u8>> = HashMap::new();
    var_names.insert("Main.counter".to_string(), vec![0, 0]);
    var_names.insert("Main.mi_uint".to_string(), vec![0, 0]);
    var_names.insert("Main.mb_bool".to_string(), vec![1]);

    //read data by name
    let iterations = 10;
    let mut results: Vec<HashMap<String, WriteResponse>> = Vec::new();
    for _ in 0..iterations {
        match client.sumup_write_by_name(var_names.clone()) {
            Ok(r) => {
                results.push(r);
            }
            Err(e) => {
                if e.is::<AdsError>() {
                    if let Some(e) = e.downcast_ref::<AdsError>() {
                        println!("Some Ads Error{:?}", e);
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

    for r in results {
        println!("{:?}", r);
    }

    //Release handle if not needed anymore
    //let result = client.release_handle(var);
    //println!("{:?}", result.unwrap());
}
