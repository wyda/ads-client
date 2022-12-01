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

    //var name and length
    let var = "Main.counter";
    let len = 2;

    //read data by name
    let result = client.read_by_name(var, len);
    println!("\n{:?}", result);
    println!(
        "\n{:?}",
        result.unwrap().data.as_slice().read_u16::<LittleEndian>()
    );

    //Release handle if not needed anymore
    let result = client.release_handle(var);
    println!("{:?}", result);
}
