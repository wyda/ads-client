use ads_client::client::Client;
use ads_proto::proto::ams_address::{AmsAddress, AmsNetId};
use byteorder::{LittleEndian, ReadBytesExt};
use std::net::Ipv4Addr;

fn main() {
    //Create client
    let ams_address = AmsAddress::new(AmsNetId::new(192, 168, 0, 150, 1, 1), 851);
    let ipv4 = Ipv4Addr::new(192, 168, 0, 150);
    let mut client = Client::new(ams_address, ipv4);
    //Connect client
    client.connect().expect("Failed to connect!");

    //Get a var handle with read_write
    let var_name = "Main.counter";
    let handle = client
        .read_write(0, 4, var_name.as_bytes().to_vec())
        .unwrap();
    let handle = handle.data.as_slice().read_u16::<LittleEndian>().unwrap();
    println!("The var hadle for {:?} is {:?}", var_name, handle);
}
