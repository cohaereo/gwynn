use anyhow::Context;
use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    collections::HashSet,
    io::{Read, Seek},
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone)]
pub enum Object {
    Code {
        argcount: u32,
        posonlyargcount: u32,
        kwonlyargcount: u32,
        stacksize: u32,
        flags: u32,
        code: Box<Object>,
        constants: Box<Object>,
        names: Box<Object>,
        localsplusnames: Box<Object>,
        localspluskinds: Box<Object>,
        filename: Box<Object>,
        name: String,
        qualname: Box<Object>,
        firstlineno: u32,
        linetable: Box<Object>,
        exceptiontable: Box<Object>,
    },
    Tuple(Vec<Object>),
    List(Vec<Object>),
    Set(Vec<Object>),

    Ref(u32),

    None,
    Null,
    Ellipsis,
    Bool(bool),
    ByteString(Vec<u8>),
    String(String),
    Long(Vec<u16>),
    Int(u32),
    Float(f64),
    Complex(f64, f64),
}

impl Object {
    pub fn as_bytestring(&self) -> Option<&[u8]> {
        match self {
            Object::ByteString(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Object::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn to_string(&self) -> Option<String> {
        self.as_string().map(|s| s.to_string())
    }
}

pub struct ObjectReader<R: Read + Seek> {
    r: R,
    pub refs: Vec<Object>,
}

impl<R: Read + Seek> ObjectReader<R> {
    pub fn new(r: R) -> Self {
        Self {
            r,
            refs: Vec::new(),
        }
    }
}

impl<R: Read + Seek> Deref for ObjectReader<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

impl<R: Read + Seek> DerefMut for ObjectReader<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.r
    }
}

pub fn read_obj<R: Read + Seek>(reader: &mut ObjectReader<R>) -> anyhow::Result<Object> {
    const TYPE_NULL: u8 = b'0';
    const TYPE_NONE: u8 = b'N';
    const TYPE_FALSE: u8 = b'F';
    const TYPE_TRUE: u8 = b'T';
    const TYPE_STOPITER: u8 = b'S';
    const TYPE_ELLIPSIS: u8 = b'.';
    const TYPE_INT: u8 = b'i';
    const TYPE_FLOAT: u8 = b'f';
    const TYPE_BINARY_FLOAT: u8 = b'g';
    const TYPE_COMPLEX: u8 = b'x';
    const TYPE_BINARY_COMPLEX: u8 = b'y';
    const TYPE_LONG: u8 = b'l';
    const TYPE_STRING: u8 = b's';
    const TYPE_INTERNED: u8 = b't';
    const TYPE_REF: u8 = b'r';
    const TYPE_TUPLE: u8 = b'(';
    const TYPE_LIST: u8 = b'[';
    const TYPE_DICT: u8 = b'{';
    const TYPE_CODE: u8 = b'c';
    const TYPE_UNICODE: u8 = b'u';
    const TYPE_UNKNOWN: u8 = b'?';
    const TYPE_SET: u8 = b'<';
    const TYPE_FROZENSET: u8 = b'>';
    const TYPE_ASCII: u8 = b'a';
    const TYPE_ASCII_INTERNED: u8 = b'A';
    const TYPE_SMALL_TUPLE: u8 = b')';
    const TYPE_SHORT_ASCII: u8 = b'z';
    const TYPE_SHORT_ASCII_INTERNED: u8 = b'Z';

    let t = reader.read_u8()?;
    let add_ref = t & 0x80 != 0;
    let t = t & 0x7F;
    let obj = match t {
        TYPE_BINARY_COMPLEX => {
            let u0 = reader.read_f64::<LittleEndian>()?;
            let u1 = reader.read_f64::<LittleEndian>()?;
            Ok(Object::Complex(u0, u1))
        }
        TYPE_LONG => {
            let m_size = reader.read_i32::<LittleEndian>()?;
            let actual_size = if m_size >= 0 { m_size } else { -m_size };
            let mut vec = Vec::with_capacity(actual_size as usize);
            for _ in 0..actual_size {
                vec.push(reader.read_u16::<LittleEndian>()?);
            }
            Ok(Object::Long(vec))
        }
        TYPE_INT => {
            let v = reader.read_u32::<LittleEndian>()?;
            Ok(Object::Int(v))
        }
        TYPE_BINARY_FLOAT => {
            let f = reader.read_f64::<LittleEndian>()?;
            Ok(Object::Float(f))
        }
        TYPE_STRING => {
            let l = reader.read_u32::<LittleEndian>()?;
            let mut buf = vec![0; l as usize];
            reader.read_exact(&mut buf)?;
            Ok(Object::ByteString(buf))
        }
        TYPE_SHORT_ASCII | TYPE_SHORT_ASCII_INTERNED => {
            let l = reader.read_u8()?;
            let mut buf = vec![0; l as usize];
            reader.read_exact(&mut buf)?;
            Ok(Object::String(String::from_utf8_lossy(&buf).to_string()))
        }
        TYPE_ASCII | TYPE_ASCII_INTERNED => {
            let l = reader.read_u32::<LittleEndian>()?;
            let mut buf = vec![0; l as usize];
            reader.read_exact(&mut buf)?;
            Ok(Object::String(String::from_utf8_lossy(&buf).to_string()))
        }
        TYPE_UNICODE => {
            let l = reader.read_u32::<LittleEndian>()?;
            let mut buf = vec![0; l as usize];
            reader.read_exact(&mut buf)?;
            Ok(Object::String(String::from_utf8_lossy(&buf).to_string()))
        }
        // TYPE_INTERNED => {
        //     let l = reader.read_u32::<LittleEndian>()?;
        //     let mut buf = vec![0; l as usize];
        //     reader.read_exact(&mut buf)?;
        //     Ok(buf)
        // }
        TYPE_TUPLE | TYPE_LIST => {
            let l = reader.read_u32::<LittleEndian>()?;
            let objs = (0..l)
                .map(|_| read_obj(reader))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Object::Tuple(objs))
        }
        TYPE_SMALL_TUPLE => {
            let l = reader.read_u8()?;
            let objs = (0..l)
                .map(|_| read_obj(reader))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Object::Tuple(objs))
        }
        // TYPE_FLOAT => {
        //     let l = reader.read_u8()?;
        //     reader.seek_relative(l as i64)?;
        //     Ok(vec![])
        // }
        // TYPE_COMPLEX => {
        //     let l = reader.read_u8()?;
        //     reader.seek_relative(l as i64)?;
        //     let l = reader.read_u8()?;
        //     reader.seek_relative(l as i64)?;
        //     Ok(vec![])
        // }
        TYPE_REF => {
            let r = reader.read_u32::<LittleEndian>()?;
            Ok(Object::Ref(r))
        }
        TYPE_FROZENSET | TYPE_SET => {
            let n = reader.read_u32::<LittleEndian>()?;
            let objs = (0..n)
                .map(|_| read_obj(reader))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Object::Set(objs))
        }
        TYPE_NONE => Ok(Object::None),
        TYPE_TRUE => Ok(Object::Bool(true)),
        TYPE_FALSE => Ok(Object::Bool(false)),
        TYPE_ELLIPSIS => Ok(Object::Ellipsis),
        TYPE_NULL => Ok(Object::Null),
        TYPE_CODE => {
            let argcount = reader.read_u32::<LittleEndian>()?; // argcount
            let posonlyargcount = reader.read_u32::<LittleEndian>()?;
            let kwonlyargcount = reader.read_u32::<LittleEndian>()?;
            let stacksize = reader.read_u32::<LittleEndian>()?;
            let flags = reader.read_u32::<LittleEndian>()?;
            let code = read_obj(reader)?; // code
            let file_name_consts = read_obj(reader)?; // consts
            let names = read_obj(reader)?; // names
            let localsplusnames = read_obj(reader)?; // localsplusnames
            let localspluskinds = read_obj(reader)?; // localspluskinds
            let filename = read_obj(reader)?; // filename
            let name = read_obj(reader)?; // name
            let qualname = read_obj(reader)?; // qualname
            let firstlineno = reader.read_u32::<LittleEndian>()?; // firstlineno
            let linetable = read_obj(reader)?; // linetable
            let exceptiontable = read_obj(reader)?; // exceptiontable

            Ok(Object::Code {
                argcount,
                posonlyargcount,
                kwonlyargcount,
                stacksize,
                flags,
                code: Box::new(code),
                constants: Box::new(file_name_consts),
                names: Box::new(names),
                localsplusnames: Box::new(localsplusnames),
                localspluskinds: Box::new(localspluskinds),
                filename: Box::new(filename),
                name: name.to_string().unwrap_or_else(|| "<unknown>".to_string()),
                qualname: Box::new(qualname),
                firstlineno,
                linetable: Box::new(linetable),
                exceptiontable: Box::new(exceptiontable),
            })
        }
        u => Err(anyhow::anyhow!(
            "Unknown obj type: {u} ('{}') at 0x{:X}",
            u as char,
            reader.stream_position()?
        )),
    };

    if let Ok(obj) = &obj
        && add_ref
    {
        reader.refs.push(obj.clone());
    }

    obj
}
