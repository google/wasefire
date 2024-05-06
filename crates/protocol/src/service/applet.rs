// Copyright 2024 Google LLC
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

use sealed::sealed;
use wasefire_error::{Code, Error};

use crate::writer::Writer;
use crate::Reader;

#[derive(Debug)]
pub struct Request<'a> {
    pub applet_id: AppletId,
    pub request: &'a [u8],
}

#[sealed]
impl<'a> crate::Serializable<'a> for Request<'a> {
    fn serialize(&self, writer: &mut Writer<'a>) {
        self.applet_id.serialize(writer);
        writer.put_share(self.request);
    }

    fn deserialize(reader: &mut Reader<'a>) -> Result<Self, Error> {
        let applet_id = AppletId::deserialize(reader)?;
        let request = reader.get_all();
        Ok(Request { applet_id, request })
    }
}

#[derive(Debug)]
pub struct Response<'a> {
    pub response: Option<&'a [u8]>,
}

#[sealed]
impl<'a> crate::Serializable<'a> for Response<'a> {
    fn serialize(&self, writer: &mut Writer<'a>) {
        match self.response {
            None => writer.put_u8(0x00),
            Some(response) => {
                writer.put_u8(0x01);
                writer.put_share(response);
            }
        }
    }

    fn deserialize(reader: &mut Reader<'a>) -> Result<Self, Error> {
        let tag = reader.get_u8()?;
        let response = match tag {
            0x00 => None,
            0x01 => Some(reader.get_all()),
            _ => return Err(Error::user(Code::InvalidArgument)),
        };
        Ok(Response { response })
    }
}

#[derive(Debug)]
pub struct AppletId;

#[sealed]
impl<'a> crate::Serializable<'a> for AppletId {
    fn serialize(&self, writer: &mut Writer<'a>) {
        let AppletId = *self;
        writer.put_u8(0x00);
    }

    fn deserialize(reader: &mut Reader<'a>) -> Result<Self, Error> {
        Error::user(Code::InvalidArgument).check(reader.get_u8()? == 0x00)?;
        Ok(AppletId)
    }
}

#[derive(Debug)]
pub struct Tunnel<'a> {
    pub applet_id: AppletId,
    pub delimiter: &'a [u8],
}

#[sealed]
impl<'a> crate::Serializable<'a> for Tunnel<'a> {
    fn serialize(&self, writer: &mut Writer<'a>) {
        self.applet_id.serialize(writer);
        writer.put_share(self.delimiter);
    }

    fn deserialize(reader: &mut Reader<'a>) -> Result<Self, Error> {
        let applet_id = AppletId::deserialize(reader)?;
        let delimiter = reader.get_all();
        Ok(Tunnel { applet_id, delimiter })
    }
}
