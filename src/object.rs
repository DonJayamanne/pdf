//! Runtime representation of a PDF file.

use std::vec::Vec;
use err::*;


use std;
use std::str::from_utf8;
use std::fmt::{Display, Formatter};
use std::collections::HashMap;


/* Objects */
pub struct IndirectObject {
    pub id: ObjectId,
    pub object: Object,
}

#[derive(Clone, Debug)]
pub enum Object {
    Integer (i32),
    RealNumber (f32),
    Boolean (bool),
    String (Vec<u8>),
    HexString (Vec<u8>), // each byte is 0-15
    Stream (Stream),
    Dictionary (Dictionary),
    Array (Vec<Object>),
    Reference (ObjectId),
    Name (String),
    Null,
}

#[derive(Clone, Debug)]
pub struct Dictionary (pub HashMap<String, Object>);

#[derive(Clone, Debug)]
pub struct Stream {
    pub dictionary: Dictionary,
    pub content: Vec<u8>,
}

#[derive(Copy, Clone, Debug)]
pub struct ObjectId {
    pub obj_nr: u32,
    pub gen_nr: u16,
}

impl Dictionary {
    pub fn new() -> Dictionary {
        Dictionary (HashMap::new())
    }
    pub fn get<'a, K>(&'a self, key: K) -> Result<&'a Object>
        where K: Into<String>
    {
        let key = key.into();
        self.0.get(&key).ok_or(ErrorKind::NotFound {word: key}.into())
    }
    pub fn set<K, V>(&mut self, key: K, value: V)
		where K: Into<String>,
		      V: Into<Object>
	{
		let _ = self.0.insert(key.into(), value.into());
	}

    /// Mostly used for debugging. If type is not specified, it will return Ok(()).
    pub fn expect_type<K>(&self, type_name: K) -> Result<()>
        where K: Into<String>
    {
        let type_name = type_name.into();
        match self.get("Type") {
            Err(_) => Ok(()),
            Ok(&Object::Name (ref name)) => {
                if *name == *type_name {
                    Ok(())
                } else {
                    bail!("Expected type {}, found type {}.", type_name, name)
                }
            }
            _ => bail!("???"),
        }
    }
}

impl Object {
    pub fn as_integer(&self) -> Result<i32> {
        match self {
            &Object::Integer (n) => Ok(n),
            _ => {
                // Err (ErrorKind::WrongObjectType.into()).chain_err(|| ErrorKind::ExpectedType {expected: "Reference"})
                Err (ErrorKind::WrongObjectType.into())
            }
        }
    }
    pub fn borrow_array(&self) -> Result<&Vec<Object>> {
        match self {
            &Object::Array (ref v) => Ok(v),
            _ => Err (ErrorKind::WrongObjectType.into())
        }
    }
    pub fn borrow_integer_array(&self) -> Result<Vec<i32>> {
        self.borrow_array()?.iter()
            .map(|x| Ok(x.as_integer()?)).collect::<Result<Vec<_>>>()
    }

    pub fn borrow_dictionary(&self) -> Result<&Dictionary> {
        match self {
            &Object::Dictionary (ref dict) => Ok(dict),
            _ => Err (ErrorKind::WrongObjectType.into())
        }
    }

    pub fn borrow_stream(&self) -> Result<&Stream> {
        match self {
            &Object::Stream (ref s) => Ok(s),
            _ => Err (ErrorKind::WrongObjectType.into()),
        }
    }

    pub fn as_array(self) -> Result<Vec<Object>> {
        match self {
            Object::Array (v) => Ok(v),
            _ => Err (ErrorKind::WrongObjectType.into())
        }
    }
    pub fn as_integer_array(self) -> Result<Vec<i32>> {
        self.as_array()?.iter()
            .map(|x| Ok(x.as_integer()?)).collect::<Result<Vec<_>>>()
    }

    pub fn as_dictionary(self) -> Result<Dictionary> {
        match self {
            Object::Dictionary (dict) => Ok(dict),
            _ => Err (ErrorKind::WrongObjectType.into())
        }
    }

    pub fn as_stream(self) -> Result<Stream> {
        match self {
            Object::Stream (s) => Ok(s),
            _ => Err (ErrorKind::WrongObjectType.into()),
        }
    }

}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            &Object::Integer(n) => write!(f, "{}", n),
            &Object::RealNumber(n) => write!(f, "{}", n),
            &Object::Boolean(b) => write!(f, "{}", if b {"true"} else {"false"}),
            &Object::String (ref s) => {
                let decoded = from_utf8(s);
                match decoded {
                    Ok(decoded) => write!(f, "({})", decoded),
                    Err(_) => {
                        // Write out bytes as numbers.
                        write!(f, "encoded(")?;
                        for c in s {
                            write!(f, "{},", c)?;
                        }
                        write!(f, ")")
                    }
                }
            },
            &Object::HexString (ref s) => {
                for c in s {
                    write!(f, "{},", c)?;
                }
                Ok(())
            }
            &Object::Stream (ref stream) => {
                write!(f, "{}", stream)
            }
            &Object::Dictionary(Dictionary(ref d)) => {
                write!(f, "<< ")?;
                for e in d {
                    write!(f, "/{} {}", e.0, e.1)?;
                }
                write!(f, ">>\n")
            },
            &Object::Array(ref a) => {
                write!(f, "[")?;
                for e in a {
                    write!(f, "{} ", e)?;
                }
                write!(f, "]")
            },
            &Object::Reference (ObjectId {obj_nr, gen_nr}) => {
                write!(f, "{} {} R", obj_nr, gen_nr)
            },
            &Object::Name (ref name) => write!(f, "/{}", name),
            &Object::Null => write!(f, "Null"),
        }
    }
}

impl Display for Stream {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let decoded = from_utf8(&self.content);
        match decoded {
            Ok(decoded) => write!(f, "stream\n{}\nendstream\n", decoded),
            Err(_) => {
                // Write out bytes as numbers.
                write!(f, "stream\n{:?}\nendstream\n", self.content)
            }
        }
    }
}

