// This file is generated by rust-protobuf 2.25.2. Do not edit
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `envoy/type/v3/semantic_version.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
// const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_25_2;

#[derive(PartialEq,Clone,Default)]
pub struct SemanticVersion {
    // message fields
    pub major_number: u32,
    pub minor_number: u32,
    pub patch: u32,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

impl<'a> ::std::default::Default for &'a SemanticVersion {
    fn default() -> &'a SemanticVersion {
        <SemanticVersion as ::protobuf::Message>::default_instance()
    }
}

impl SemanticVersion {
    pub fn new() -> SemanticVersion {
        ::std::default::Default::default()
    }

    // uint32 major_number = 1;


    pub fn get_major_number(&self) -> u32 {
        self.major_number
    }
    pub fn clear_major_number(&mut self) {
        self.major_number = 0;
    }

    // Param is passed by value, moved
    pub fn set_major_number(&mut self, v: u32) {
        self.major_number = v;
    }

    // uint32 minor_number = 2;


    pub fn get_minor_number(&self) -> u32 {
        self.minor_number
    }
    pub fn clear_minor_number(&mut self) {
        self.minor_number = 0;
    }

    // Param is passed by value, moved
    pub fn set_minor_number(&mut self, v: u32) {
        self.minor_number = v;
    }

    // uint32 patch = 3;


    pub fn get_patch(&self) -> u32 {
        self.patch
    }
    pub fn clear_patch(&mut self) {
        self.patch = 0;
    }

    // Param is passed by value, moved
    pub fn set_patch(&mut self, v: u32) {
        self.patch = v;
    }
}

impl ::protobuf::Message for SemanticVersion {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.major_number = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.minor_number = tmp;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint32()?;
                    self.patch = tmp;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.major_number != 0 {
            my_size += ::protobuf::rt::value_size(1, self.major_number, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.minor_number != 0 {
            my_size += ::protobuf::rt::value_size(2, self.minor_number, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.patch != 0 {
            my_size += ::protobuf::rt::value_size(3, self.patch, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::ProtobufResult<()> {
        if self.major_number != 0 {
            os.write_uint32(1, self.major_number)?;
        }
        if self.minor_number != 0 {
            os.write_uint32(2, self.minor_number)?;
        }
        if self.patch != 0 {
            os.write_uint32(3, self.patch)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> SemanticVersion {
        SemanticVersion::new()
    }

    fn descriptor_static() -> &'static ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::LazyV2<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                "major_number",
                |m: &SemanticVersion| { &m.major_number },
                |m: &mut SemanticVersion| { &mut m.major_number },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                "minor_number",
                |m: &SemanticVersion| { &m.minor_number },
                |m: &mut SemanticVersion| { &mut m.minor_number },
            ));
            fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                "patch",
                |m: &SemanticVersion| { &m.patch },
                |m: &mut SemanticVersion| { &mut m.patch },
            ));
            ::protobuf::reflect::MessageDescriptor::new_pb_name::<SemanticVersion>(
                "SemanticVersion",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static SemanticVersion {
        static instance: ::protobuf::rt::LazyV2<SemanticVersion> = ::protobuf::rt::LazyV2::INIT;
        instance.get(SemanticVersion::new)
    }
}

impl ::protobuf::Clear for SemanticVersion {
    fn clear(&mut self) {
        self.major_number = 0;
        self.minor_number = 0;
        self.patch = 0;
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for SemanticVersion {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for SemanticVersion {
    fn as_ref(&self) -> ::protobuf::reflect::ReflectValueRef {
        ::protobuf::reflect::ReflectValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n$envoy/type/v3/semantic_version.proto\x12\renvoy.type.v3\x1a\x1dudpa/a\
    nnotations/status.proto\x1a!udpa/annotations/versioning.proto\"\x90\x01\
    \n\x0fSemanticVersion\x12!\n\x0cmajor_number\x18\x01\x20\x01(\rR\x0bmajo\
    rNumber\x12!\n\x0cminor_number\x18\x02\x20\x01(\rR\x0bminorNumber\x12\
    \x14\n\x05patch\x18\x03\x20\x01(\rR\x05patch:!\x9a\xc5\x88\x1e\x1c\n\x1a\
    envoy.type.SemanticVersionB=\n\x1bio.envoyproxy.envoy.type.v3B\x14Semant\
    icVersionProtoP\x01\xba\x80\xc8\xd1\x06\x02\x10\x02b\x06proto3\
";

static file_descriptor_proto_lazy: ::protobuf::rt::LazyV2<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::LazyV2::INIT;

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    file_descriptor_proto_lazy.get(|| {
        parse_descriptor_proto()
    })
}
