// Copyright (c) 2018 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Binary protocol encoder and decoder.
//!
//! This module contains functions and types for serializing and deserializing Srv-Rpc
//! wire messages.
//!
//! # Protocol details
//!
//! * Header Segment (32-bits)
//!   * is_txn (1-bit)
//!   * flags (5-bits)
//!   * message_id_len (6-bits)
//!   * body_len (20-bits)
//! * Transaction Segment (32-bits, optional)
//!   * is_response (1-bit)
//!   * is_partial (1-bit)
//!   * txn_identifier (30-bits)
//! * Message Identifier Segment (variable length)
//! * Message Body Segment (variable length)

use std::fmt;
use std::io::{self, Cursor};
use std::ops::{Deref, DerefMut};

use bytes::{BigEndian, Buf, BufMut, Bytes, BytesMut};
use futures;
use protobuf;
use tokio::net::TcpStream;
use tokio_io::codec::{Decoder, Encoder, Framed};

const BODY_LEN_MASK: u32 = 0xFFFFF;
const HEADER_LEN: usize = 4;
const MESSAGE_ID_MASK: u32 = 0x3F;
const MESSAGE_ID_OFFSET: u32 = 20;
const TXN_LEN: usize = 4;
const TXN_OFFSET: u32 = 31;

const RESPONSE_OFFSET: u32 = 31;
const RESPONSE_MASK: u32 = 0x1;
const PARTIAL_OFFSET: u32 = 30;
const PARTIAL_MASK: u32 = 0x1;

lazy_static! {
    static ref MAX_TXN_ID: u32 = {
        2u32.pow(30)
    };
}

pub type SrvStream = Framed<TcpStream, SrvCodec>;
pub type SrvSink = futures::stream::SplitSink<SrvStream>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SrvTxn(u32);

impl SrvTxn {
    pub fn new(id: u32) -> Self {
        assert!(
            id <= *MAX_TXN_ID,
            "cannot construct transaction with id larger than MAX_TXN_ID"
        );
        SrvTxn(id)
    }

    pub fn next_id(mut id: u32) -> u32 {
        while id != *MAX_TXN_ID && id != 0 {
            id += 1;
        }
        id
    }

    pub fn is_complete(&self) -> bool {
        ((self.0 >> PARTIAL_OFFSET) & PARTIAL_MASK) == 0
    }

    pub fn is_response(&self) -> bool {
        ((self.0 >> RESPONSE_OFFSET) & RESPONSE_MASK) > 0
    }

    pub fn set_complete(&mut self, flag: bool) {
        let value = if flag { 0 } else { 1 };
        self.0 = (value << PARTIAL_OFFSET) | self.0;
    }

    pub fn set_response(&mut self, flag: bool) {
        let value = if flag { 1 } else { 0 };
        self.0 = (value << RESPONSE_OFFSET) | self.0;
    }
}

impl From<u32> for SrvTxn {
    fn from(value: u32) -> Self {
        SrvTxn(value)
    }
}

impl fmt::Display for SrvTxn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SrvTxn[id: {}, is_complete: {}, is_response: {}]",
            self.0,
            self.is_complete(),
            self.is_response(),
        )
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SrvHeader(u32);

impl SrvHeader {
    pub fn new(body_len: u32, message_id_len: u32, is_txn: bool) -> Self {
        let txn_value = if is_txn { 1 } else { 0 };
        let value = (txn_value << TXN_OFFSET) | (message_id_len << MESSAGE_ID_OFFSET) | body_len;
        SrvHeader(value)
    }

    #[inline]
    pub fn body_len(&self) -> usize {
        (self.0 & BODY_LEN_MASK) as usize
    }

    #[inline]
    pub fn message_id_len(&self) -> usize {
        ((self.0 >> MESSAGE_ID_OFFSET) & MESSAGE_ID_MASK) as usize
    }

    #[inline]
    pub fn is_transaction(&self) -> bool {
        match self.0 >> TXN_OFFSET {
            1 => true,
            0 => false,
            _ => unreachable!(),
        }
    }
}

impl From<u32> for SrvHeader {
    fn from(value: u32) -> Self {
        SrvHeader(value)
    }
}

impl fmt::Display for SrvHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SrvHeader[body_len: {}, message_id_len: {}, is_txn: {}]",
            self.body_len(),
            self.message_id_len(),
            self.is_transaction()
        )
    }
}

#[derive(Clone, Debug)]
pub struct SrvMessage<T> {
    pub transaction: Option<SrvTxn>,
    inner: T,
}

impl<T> SrvMessage<T>
where
    T: protobuf::MessageStatic,
{
    pub fn build(body: &Bytes, txn: Option<SrvTxn>) -> Result<Self, protobuf::ProtobufError> {
        let payload = protobuf::parse_from_carllerche_bytes::<T>(body)?;
        Ok(SrvMessage {
            inner: payload,
            transaction: txn,
        })
    }

    pub fn new() -> Self {
        SrvMessage {
            inner: T::new(),
            transaction: None,
        }
    }

    pub fn message_id(&self) -> &str {
        self.inner.descriptor().name()
    }

    pub fn reply_for(&mut self, mut txn: SrvTxn, complete: bool) {
        txn.set_response(true);
        txn.set_complete(complete);
        self.transaction = Some(txn);
    }
}

impl<T> Deref for SrvMessage<T>
where
    T: protobuf::Message,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for SrvMessage<T>
where
    T: protobuf::Message,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Clone, Debug)]
pub struct SrvWireMessage {
    header: SrvHeader,
    transaction: Option<SrvTxn>,
    message_id: String,
    body: Bytes,
}

impl SrvWireMessage {
    pub fn new<T>(msg: SrvMessage<T>) -> Self
    where
        T: protobuf::MessageStatic + protobuf::MessageStatic,
    {
        let body = Bytes::from(msg.write_to_bytes().unwrap());
        let message_id = msg.descriptor().name().to_string();
        SrvWireMessage {
            header: SrvHeader::new(
                body.len() as u32,
                message_id.len() as u32,
                msg.transaction.is_some(),
            ),
            transaction: msg.transaction,
            message_id: message_id,
            body: body,
        }
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }

    pub fn header(&self) -> SrvHeader {
        self.header
    }

    pub fn is_complete(&self) -> bool {
        match self.transaction {
            Some(txn) => txn.is_complete(),
            None => false,
        }
    }

    pub fn is_response(&self) -> bool {
        match self.transaction {
            Some(txn) => txn.is_response(),
            None => false,
        }
    }

    pub fn is_transaction(&self) -> bool {
        self.transaction.is_some()
    }

    pub fn message_id(&self) -> &str {
        &self.message_id
    }

    pub fn parse<T>(&self) -> Result<SrvMessage<T>, protobuf::ProtobufError>
    where
        T: protobuf::Message + protobuf::MessageStatic,
    {
        SrvMessage::build(&self.body, self.transaction)
    }

    pub fn size(&self) -> usize {
        let mut size = HEADER_LEN;
        if self.transaction.is_some() {
            size += TXN_LEN;
        }
        size += self.message_id().len();
        size += self.body().len();
        size
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SrvCodec(());

impl SrvCodec {
    /// Creates a new `SrvCodec` for shipping around `SrvWireMessage`s.
    pub fn new() -> SrvCodec {
        SrvCodec(())
    }
}

impl Decoder for SrvCodec {
    type Item = SrvWireMessage;
    type Error = io::Error;

    fn decode(&mut self, bytes: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {
        trace!("Decoding SrvWireMessage\n  -> Bytes: {:?}", bytes);
        if bytes.len() < HEADER_LEN {
            return Ok(None);
        }
        let mut buf = Cursor::new(bytes);
        let header = SrvHeader(buf.get_u32::<BigEndian>());
        trace!("  -> SrvHeader: {}", header);
        let mut txn: Option<SrvTxn> = None;
        if header.is_transaction() {
            if buf.remaining() < TXN_LEN {
                return Ok(None);
            }
            let t = SrvTxn(buf.get_u32::<BigEndian>());
            trace!("  -> SrvTxn: {}", t);
            txn = Some(t);
        }
        if buf.remaining() < (header.message_id_len() + header.body_len()) {
            // Not enough bytes to read message_id and body
            return Ok(None);
        }
        // I can probably use a single buffer for this instead of allocating two everytime we want
        // to process a message.
        let mut message_id_buf: Vec<u8> = vec![0; header.message_id_len()];
        let mut body_buf: Vec<u8> = vec![0; header.body_len()];
        buf.copy_to_slice(&mut message_id_buf);
        buf.copy_to_slice(&mut body_buf);
        let message_id = String::from_utf8(message_id_buf).unwrap();
        let position = buf.position() as usize;
        let bytes = buf.into_inner();
        bytes.split_to(position);
        Ok(Some(SrvWireMessage {
            header: header,
            transaction: txn,
            message_id: message_id,
            body: Bytes::from(body_buf),
        }))
    }
}

impl Encoder for SrvCodec {
    type Item = SrvWireMessage;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        buf.reserve(msg.size());
        buf.put_u32::<BigEndian>(msg.header().0);
        if let Some(txn) = msg.transaction {
            buf.put_u32::<BigEndian>(txn.0);
        }
        buf.put_slice(msg.message_id().as_bytes());
        buf.put_slice(msg.body());
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use protocols;

    #[test]
    fn test_header_pack_unpack() {
        let body_value = 305888;
        let message_id_value = 40;
        let header = SrvHeader::new(body_value, message_id_value, true);
        assert_eq!(header.body_len(), body_value);
        assert_eq!(header.message_id_len(), message_id_value);
        assert_eq!(header.is_transaction(), true);
    }


    #[test]
    #[should_panic]
    fn test_txn_partial_bit_reserved() {
        SrvTxn::new(2u32.pow(32));
    }

    #[test]
    #[should_panic]
    fn test_txn_response_bit_reserved() {
        SrvTxn::new(2u32.pow(31));
    }

    #[test]
    fn test_txn_pack_unpack() {
        let mut txn = SrvTxn::new(*MAX_TXN_ID);
        assert_eq!(txn.is_complete(), false);
        assert_eq!(txn.is_response(), false);

        txn.set_complete(true);
        assert_eq!(txn.is_complete(), true);
        assert_eq!(txn.is_response(), false);

        txn.set_response(true);
        assert_eq!(txn.is_complete(), true);
        assert_eq!(txn.is_response(), true);

        txn.set_complete(false);
        assert_eq!(txn.is_complete(), false);
        assert_eq!(txn.is_response(), true);
    }

    #[test]
    fn test_codec() {
        let mut codec = SrvCodec::new();
        let mut inner = protocols::main::HelloWorld::new();
        inner.set_page(3);
        inner.set_what("this".to_string());
        let msg = SrvWireMessage::from(inner);
        let mut buf = BytesMut::new();
        codec.encode(msg.clone(), &mut buf).unwrap();
        let decoded = codec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(decoded.header(), msg.header());
        assert_eq!(decoded.message_id(), msg.message_id());
        assert_eq!(decoded.transaction(), msg.transaction());
        assert_eq!(decoded.body(), msg.body());
    }
}
