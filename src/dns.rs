use std::net::{SocketAddr, UdpSocket, IpAddr, ToSocketAddrs};
use std::time::Duration;
use rand;
use trust_dns::op::{Message, MessageType, OpCode, Query};
use trust_dns::rr::domain::Name;
use trust_dns::rr::record_type::RecordType;
use trust_dns::serialize::binary::*;
use trust_dns::proto::error::ProtoError;

#[derive(Debug)]
pub enum Error {
    Proto(ProtoError),
    IO(std::io::Error),
    Encoding(ProtoError),
    Decoding(ProtoError),
    Network(std::io::Error),
    Sending(std::io::Error),
    Receving(std::io::Error),
}

impl From<ProtoError> for Error {
    fn from(err: ProtoError) -> Self {
        Error::Proto(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(err)
    }
}

pub fn resolve<'a>(dns_server: impl ToSocketAddrs, domain_name: &str) -> Result<Message, Error> {
    let mut request_as_byte: Vec<u8> = Vec::with_capacity(512); // length of 0 and capacity of 512
    let mut response_as_byte: Vec<u8> = vec![0; 512]; // length of 512 and capacity of 512
    let domain_name = Name::from_ascii(domain_name)?;

    let mut request = Message::new();
    request
        .set_id(rand::random::<u16>())
        .set_message_type(MessageType::Query)
        .add_query(Query::query(domain_name, RecordType::A))
        .set_op_code(OpCode::Query)
        .set_recursion_desired(true);

    let mut encoder = BinEncoder::new(&mut request_as_byte);
    request.emit(&mut encoder)
        .map_err(Error::Encoding)?;

    // Listen all addresses on a random port selected by the OS
    let localhost = UdpSocket::bind("0.0.0.0:0")?;
    let timeout = Duration::from_secs(3);
    localhost.set_read_timeout(Some(timeout))?;
    localhost.set_nonblocking(false)
        .map_err(Error::Network)?;

    let _amt = localhost
        .send_to(&request_as_byte, dns_server)
        .map_err(Error::Sending)?;

    let (_amt, _remote) = localhost
        .recv_from(&mut response_as_byte)
        .map_err(Error::Receving)?;

    let response = Message::from_vec(&response_as_byte)
        .map_err(Error::Decoding)?;        

    Ok(response)
}
