use std::{net::SocketAddr, str::FromStr};

use clap::Parser;
use smoltcp::phy::TapInterface;
use url::Url;

mod dns;
mod ethernet;
mod http;

#[derive(Debug)]
struct TapInterfaceAux(TapInterface);

impl FromStr for TapInterfaceAux {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            0: TapInterface::new(s)?,
        })
    }
}

impl Into<TapInterface> for TapInterfaceAux {
    fn into(self) -> TapInterface {
        self.0
    }
}

#[derive(Parser, Debug)]
struct Cli {
    #[clap(long)]
    url: Url,

    #[clap(long)]
    tap_device: TapInterfaceAux,

    #[clap(long, default_value = "1.1.1.1")]
    dns_server: SocketAddr,
}

fn main() {
    let options = Cli::parse();

    if options.url.scheme() != "http" {
        eprintln!("error: only HTTP protocol supported for now...");
        return;
    }

    let tap: TapInterface = options.tap_device.into();

    let domain_name = options.url.host_str()
        .expect("domain name required");

    let answers = dns::resolve(&options.dns_server, domain_name)
        .expect("Unable to resolve the domain name");

    let addresses: Vec<std::net::IpAddr> = answers.iter().filter(|answer| {
        return answer.record_type() == trust_dns::rr::record_type::RecordType::A
    }).map(|answer| {
        let resource = answer.rdata();
        let ip = resource.to_ip_addr()
            .expect("Invalid IP address received");
        ip
    }).collect();

    if addresses.is_empty() {
        println!("There is no IP address associated");
        return;
    }

    let addr = addresses.first().unwrap();

    let mac = ethernet::MacAddress::new().into();
    http::get(tap, mac, addr, url).expect("Unable to perform GET HTTP");
}
