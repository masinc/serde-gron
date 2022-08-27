use bool_ext::BoolExt;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{ser, Serialize};
use std::{fmt::Display, io};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid root name")]
    InvalidRootName,

    #[error("Reached end of file")]
    Eof,
    #[error(transparent)]
    Serialize(serde_json::Error),
    #[error(transparent)]
    Io(io::Error),

    #[error("Error: {0}")]
    Custom(String),
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

pub fn to_string(value: &impl Serialize) -> Result<String, Error> {
    to_string_with(value, "json", FormatType::Regular)
}

pub fn to_colored_string(value: &impl Serialize) -> Result<String, Error> {
    to_string_with(value, "json", FormatType::Color)
}

pub fn to_string_with(
    value: &impl Serialize,
    root_name: impl Into<String>,
    format_type: FormatType,
) -> Result<String, Error> {
    let mut writer = vec![];
    to_writer_with(value, &mut writer, root_name, format_type)?;
    Ok(String::from_utf8(writer).unwrap())
}

pub fn to_writer(value: &impl Serialize, writer: &mut impl io::Write) -> Result<(), Error> {
    to_writer_with(value, writer, "json", FormatType::Regular)
}

pub fn to_colored_writer(value: &impl Serialize, writer: &mut impl io::Write) -> Result<(), Error> {
    to_writer_with(value, writer, "json", FormatType::Color)
}

pub fn to_writer_with(
    value: &impl Serialize,
    writer: &mut impl io::Write,
    root_name: impl Into<String>,
    format_type: FormatType,
) -> Result<(), Error> {
    match format_type {
        FormatType::Regular => {
            let mut ser = Serializer::<_, RegularFormatter>::new_with_root_name(writer, root_name);
            value.serialize(&mut ser)?;
        }
        FormatType::Color => {
            let mut ser = Serializer::<_, ColorFormatter>::new_with_root_name(writer, root_name);
            value.serialize(&mut ser)?;
        }
    };

    Ok(())
}

#[derive(Debug, Clone)]
pub enum FormatType {
    /// Non colored output
    Regular,
    /// Colored output
    Color,
}

#[derive(Debug)]
pub enum NamespaceKey {
    Array(usize),
    Object(String),
}

pub trait Formatter<W: io::Write> {
    fn write_key(&self, wriiter: &mut W, ns_root: &str, nss: &[NamespaceKey]) -> Result<(), Error>;
    fn write_key_value_delimiter(&self, wriiter: &mut W) -> Result<(), Error>;
    fn write_end_of_line(&self, writer: &mut W) -> Result<(), Error>;

    fn write_null(&self, writer: &mut W) -> Result<(), Error>;
    fn write_bool(&self, writer: &mut W, value: bool) -> Result<(), Error>;
    fn write_number<N: num::Num + Display>(&self, writer: &mut W, value: N) -> Result<(), Error>;
    fn write_string(&self, writer: &mut W, value: &str) -> Result<(), Error>;
    fn write_init_array(&self, writer: &mut W) -> Result<(), Error>;
    fn write_init_object(&self, writer: &mut W) -> Result<(), Error>;
}

#[derive(Debug)]
struct Context {
    ns_root: String,
    ns: Vec<NamespaceKey>,

    finish: bool,
}

impl Context {
    fn new() -> Context {
        Context::new_with_root_name("json")
    }

    fn new_with_root_name(name: impl Into<String>) -> Context {
        Context {
            ns_root: name.into(),
            ns: vec![],
            finish: false,
        }
    }

    fn is_root(&self) -> bool {
        self.ns.is_empty()
    }

    fn error_if_finished(&self) -> Result<(), Error> {
        (!self.finish).err_with(|| Error::Eof)
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Serializer<W, F = RegularFormatter> {
    writer: W,
    formatter: F,
    ctx: Context,
}

impl<W, F> Serializer<W, F>
where
    W: io::Write,
    F: Formatter<W> + Default,
{
    pub fn new(writer: W) -> Self {
        Self::new_with_root_name(writer, "json")
    }

    pub fn new_with_root_name(writer: W, root_name: impl Into<String>) -> Self {
        Self {
            writer,
            formatter: F::default(),
            ctx: Context::new_with_root_name(root_name),
        }
    }
}

impl<W, F> Serializer<W, F>
where
    W: io::Write,
    F: Formatter<W>,
{
    fn serialize_number<N: num::Num + Display>(&mut self, n: N) -> Result<(), Error> {
        self.ctx.error_if_finished()?;
        self.formatter
            .write_key(&mut self.writer, &self.ctx.ns_root, &self.ctx.ns)?;
        self.formatter.write_key_value_delimiter(&mut self.writer)?;
        self.formatter.write_number(&mut self.writer, n)?;
        self.formatter.write_end_of_line(&mut self.writer)?;

        if self.ctx.is_root() {
            self.ctx.finish = true;
        }

        Ok(())
    }

    fn serialize_array_init(&mut self) -> Result<(), Error> {
        self.ctx.error_if_finished()?;
        self.formatter
            .write_key(&mut self.writer, &self.ctx.ns_root, &self.ctx.ns)?;
        self.formatter.write_key_value_delimiter(&mut self.writer)?;
        self.formatter.write_init_array(&mut self.writer)?;
        self.formatter.write_end_of_line(&mut self.writer)?;

        Ok(())
    }

    fn serialize_object_init(&mut self) -> Result<(), Error> {
        self.ctx.error_if_finished()?;
        self.formatter
            .write_key(&mut self.writer, &self.ctx.ns_root, &self.ctx.ns)?;
        self.formatter.write_key_value_delimiter(&mut self.writer)?;
        self.formatter.write_init_object(&mut self.writer)?;
        self.formatter.write_end_of_line(&mut self.writer)?;

        Ok(())
    }
}

impl<'a, W: io::Write, F: Formatter<W>> ser::Serializer for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.ctx.error_if_finished()?;

        self.formatter
            .write_key(&mut self.writer, &self.ctx.ns_root, &self.ctx.ns)?;
        self.formatter.write_key_value_delimiter(&mut self.writer)?;
        self.formatter.write_bool(&mut self.writer, v)?;

        if self.ctx.is_root() {
            self.ctx.finish = true;
        }

        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_number(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_number(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_number(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_number(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_number(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_number(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_number(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_number(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_number(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.serialize_number(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.ctx.error_if_finished()?;
        self.formatter
            .write_key(&mut self.writer, &self.ctx.ns_root, &self.ctx.ns)?;
        self.formatter.write_key_value_delimiter(&mut self.writer)?;
        self.formatter.write_string(&mut self.writer, v)?;
        self.formatter.write_end_of_line(&mut self.writer)?;

        if self.ctx.is_root() {
            self.ctx.finish = true;
        }

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut seq = self.serialize_seq(Some(v.len()))?;

        for b in v {
            ser::SerializeSeq::serialize_element(&mut seq, b)?;
        }
        ser::SerializeSeq::end(seq)?;

        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.ctx.error_if_finished()?;
        self.formatter
            .write_key(&mut self.writer, &self.ctx.ns_root, &self.ctx.ns)?;
        self.formatter.write_key_value_delimiter(&mut self.writer)?;
        self.formatter.write_null(&mut self.writer)?;
        self.formatter.write_end_of_line(&mut self.writer)?;
        if self.ctx.is_root() {
            self.ctx.finish = true;
        }
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        unimplemented!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.serialize_array_init()?;
        self.ctx.ns.push(NamespaceKey::Array(0));
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.serialize_object_init()?;
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_map(Some(len))
    }
}

impl<'a, W: io::Write, F: Formatter<W>> ser::SerializeSeq for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)?;

        match self.ctx.ns.last_mut() {
            Some(v) => match v {
                NamespaceKey::Array(n) => *n += 1,
                NamespaceKey::Object(_) => unreachable!(),
            },

            None => unreachable!(),
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.ctx.ns.pop();
        Ok(())
    }
}

impl<'a, W: io::Write, F: Formatter<W>> ser::SerializeTuple for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W: io::Write, F: Formatter<W>> ser::SerializeMap for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let key = serde_json::to_string(key)
            .map_err(Error::Serialize)?
            .trim_matches('"')
            .to_string();
        self.ctx.ns.push(NamespaceKey::Object(key));

        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)?;
        self.ctx.ns.pop();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: io::Write, F: Formatter<W>> ser::SerializeStruct for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeMap::end(self)
    }
}

impl<'a, W: io::Write, F: Formatter<W>> ser::SerializeTupleStruct for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W: io::Write, F: Formatter<W>> ser::SerializeTupleVariant for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W: io::Write, F: Formatter<W>> ser::SerializeStructVariant for &'a mut Serializer<W, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeMap::end(self)
    }
}

static RE_OBJECT_KEY: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9_]*$").unwrap());
fn write_key_object(writer: &mut String, key: &str) -> Result<(), std::fmt::Error> {
    use std::fmt::Write as _;

    if RE_OBJECT_KEY.is_match(key) {
        write!(writer, ".{key}")
    } else {
        write!(writer, "[\"{key}\"]")
    }
}

#[derive(Debug, Default)]
pub struct RegularFormatter;

impl<W: io::Write> Formatter<W> for RegularFormatter {
    fn write_key(&self, writer: &mut W, ns_root: &str, nss: &[NamespaceKey]) -> Result<(), Error> {
        use std::fmt::Write as _;
        let mut res = String::new();
        res.push_str(ns_root);
        for ns in nss.iter() {
            match ns {
                NamespaceKey::Array(n) => write!(res, "[{n}]").unwrap(),
                NamespaceKey::Object(k) => write_key_object(&mut res, k).unwrap(),
            };
        }

        write!(writer, "{res}").map_err(Error::Io)
    }

    fn write_key_value_delimiter(&self, wriiter: &mut W) -> Result<(), Error> {
        write!(wriiter, " = ").map_err(Error::Io)
    }

    fn write_end_of_line(&self, writer: &mut W) -> Result<(), Error> {
        writeln!(writer, ";").map_err(Error::Io)
    }

    fn write_null(&self, writer: &mut W) -> Result<(), Error> {
        write!(writer, "null").map_err(Error::Io)
    }

    fn write_bool(&self, writer: &mut W, value: bool) -> Result<(), Error> {
        write!(writer, "{value}").map_err(Error::Io)
    }

    fn write_number<N: num::Num + Display>(&self, writer: &mut W, value: N) -> Result<(), Error> {
        write!(writer, "{value}").map_err(Error::Io)
    }

    fn write_string(&self, writer: &mut W, value: &str) -> Result<(), Error> {
        write!(writer, "\"{value}\"").map_err(Error::Io)
    }

    fn write_init_array(&self, writer: &mut W) -> Result<(), Error> {
        write!(writer, "[]").map_err(Error::Io)
    }

    fn write_init_object(&self, writer: &mut W) -> Result<(), Error> {
        write!(writer, "{{}}").map_err(Error::Io)
    }
}

#[derive(Debug, Default)]
pub struct ColorFormatter;

impl<W: io::Write> Formatter<W> for ColorFormatter {
    fn write_key(&self, writer: &mut W, ns_root: &str, nss: &[NamespaceKey]) -> Result<(), Error> {
        todo!()
    }

    fn write_key_value_delimiter(&self, wriiter: &mut W) -> Result<(), Error> {
        todo!()
    }

    fn write_end_of_line(&self, writer: &mut W) -> Result<(), Error> {
        todo!()
    }

    fn write_null(&self, writer: &mut W) -> Result<(), Error> {
        todo!()
    }

    fn write_bool(&self, writer: &mut W, value: bool) -> Result<(), Error> {
        todo!()
    }

    fn write_number<N: num::Num + Display>(&self, writer: &mut W, value: N) -> Result<(), Error> {
        todo!()
    }

    fn write_string(&self, writer: &mut W, value: &str) -> Result<(), Error> {
        todo!()
    }

    fn write_init_array(&self, writer: &mut W) -> Result<(), Error> {
        todo!()
    }

    fn write_init_object(&self, writer: &mut W) -> Result<(), Error> {
        todo!()
    }
}
