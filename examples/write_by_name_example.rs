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

    //var name and length
    let var = "Main.counter";

    //Write "1111" to var "Main.counter"
    let value: u16 = 1111;
    let data: Vec<u8> = value.to_le_bytes().to_vec();
    let write_result = client
        .write_by_name(var, data)
        .expect("Failed to write value!");
    println!("Write result -> {:?}", write_result.result);
    println!("Command ID -> {:?}", write_result.command_id);

    //var name and length
    let var = "Main.mi_uint";

    //Write "65530" to var "Main.mi_uint"
    let value: u16 = 65530;
    let data: Vec<u8> = value.to_le_bytes().to_vec();
    let write_result = client
        .write_by_name(var, data)
        .expect("Failed to write value!");
    println!("Write result -> {:?}", write_result.result);
    println!("Command ID -> {:?}", write_result.command_id);

    //Release handle if not needed anymore
    let result = client.release_handle(var);
    println!("{:?}", result);
}
