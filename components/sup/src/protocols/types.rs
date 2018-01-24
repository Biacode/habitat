// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum InstallSource {
    Ident = 0,
    Archive = 1,
}

impl ::protobuf::ProtobufEnum for InstallSource {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<InstallSource> {
        match value {
            0 => ::std::option::Option::Some(InstallSource::Ident),
            1 => ::std::option::Option::Some(InstallSource::Archive),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [InstallSource] = &[
            InstallSource::Ident,
            InstallSource::Archive,
        ];
        values
    }

    fn enum_descriptor_static(_: ::std::option::Option<InstallSource>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("InstallSource", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for InstallSource {
}

impl ::protobuf::reflect::ProtobufValue for InstallSource {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum Topology {
    Standalone = 0,
    Leader = 1,
}

impl ::protobuf::ProtobufEnum for Topology {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<Topology> {
        match value {
            0 => ::std::option::Option::Some(Topology::Standalone),
            1 => ::std::option::Option::Some(Topology::Leader),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [Topology] = &[
            Topology::Standalone,
            Topology::Leader,
        ];
        values
    }

    fn enum_descriptor_static(_: ::std::option::Option<Topology>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("Topology", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for Topology {
}

impl ::protobuf::reflect::ProtobufValue for Topology {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum UpdateStrategy {
    None = 0,
    AtOnce = 1,
    Rolling = 2,
}

impl ::protobuf::ProtobufEnum for UpdateStrategy {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<UpdateStrategy> {
        match value {
            0 => ::std::option::Option::Some(UpdateStrategy::None),
            1 => ::std::option::Option::Some(UpdateStrategy::AtOnce),
            2 => ::std::option::Option::Some(UpdateStrategy::Rolling),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [UpdateStrategy] = &[
            UpdateStrategy::None,
            UpdateStrategy::AtOnce,
            UpdateStrategy::Rolling,
        ];
        values
    }

    fn enum_descriptor_static(_: ::std::option::Option<UpdateStrategy>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("UpdateStrategy", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for UpdateStrategy {
}

impl ::protobuf::reflect::ProtobufValue for UpdateStrategy {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0btypes.proto*'\n\rInstallSource\x12\t\n\x05Ident\x10\0\x12\x0b\n\
    \x07Archive\x10\x01*&\n\x08Topology\x12\x0e\n\nStandalone\x10\0\x12\n\n\
    \x06Leader\x10\x01*3\n\x0eUpdateStrategy\x12\x08\n\x04None\x10\0\x12\n\n\
    \x06AtOnce\x10\x01\x12\x0b\n\x07Rolling\x10\x02\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
