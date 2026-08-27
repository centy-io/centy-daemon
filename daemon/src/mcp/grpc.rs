use bytes::Buf;
use color_eyre::eyre::{Result, WrapErr};
use prost::Message;
use prost_reflect::{DynamicMessage, MessageDescriptor};
use std::net::{IpAddr, SocketAddr};
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};
use tonic::transport::Endpoint;
use tonic::Status;

pub fn endpoint_for(addr: SocketAddr) -> String {
    let host = match addr.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => "127.0.0.1".to_owned(),
        IpAddr::V6(ip) if ip.is_unspecified() => "[::1]".to_owned(),
        IpAddr::V6(_) => format!("[{}]", addr.ip()),
        IpAddr::V4(_) => addr.ip().to_string(),
    };
    format!("http://{host}:{}", addr.port())
}

pub async fn unary(
    endpoint: &str,
    path: http::uri::PathAndQuery,
    request: DynamicMessage,
    response: MessageDescriptor,
) -> Result<DynamicMessage> {
    let channel = Endpoint::from_shared(endpoint.to_owned())
        .wrap_err("invalid daemon gRPC endpoint")?
        .connect()
        .await
        .wrap_err("failed to connect to daemon gRPC endpoint")?;
    tonic::client::Grpc::new(channel)
        .unary(
            tonic::Request::new(request),
            path,
            DynamicCodec { response },
        )
        .await
        .map(tonic::Response::into_inner)
        .map_err(color_eyre::Report::from)
}

#[derive(Clone)]
struct DynamicCodec {
    response: MessageDescriptor,
}

impl Codec for DynamicCodec {
    type Encode = DynamicMessage;
    type Decode = DynamicMessage;
    type Encoder = DynamicEncoder;
    type Decoder = DynamicDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        DynamicEncoder
    }

    fn decoder(&mut self) -> Self::Decoder {
        DynamicDecoder {
            descriptor: self.response.clone(),
        }
    }
}

struct DynamicEncoder;

impl Encoder for DynamicEncoder {
    type Item = DynamicMessage;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        item.encode(dst)
            .map_err(|error| Status::internal(error.to_string()))
    }
}

struct DynamicDecoder {
    descriptor: MessageDescriptor,
}

impl Decoder for DynamicDecoder {
    type Item = DynamicMessage;
    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        if !src.has_remaining() {
            return Ok(None);
        }
        DynamicMessage::decode(self.descriptor.clone(), src)
            .map(Some)
            .map_err(|error| Status::internal(error.to_string()))
    }
}
