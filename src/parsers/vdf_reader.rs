/*
 * This file is part of Steam-Art-Manager which is licensed under GNU Lesser General Public License v2.1
 * See file LICENSE or go to https://www.gnu.org/licenses/old-licenses/lgpl-2.1.en.html for full license details
 */

use serde_json::{Map, Value};

use super::reader::Reader;

/// Reads a vdf entry string to JSON.
pub fn read_vdf_string(
    reader: &mut Reader,
    magic: Option<u32>,
    strings: &Option<&mut Vec<String>>,
) -> String {
    if magic.is_some() && magic.unwrap() == 0x07564429 {
        let index: usize = reader.read_uint32(true).try_into().unwrap();
        let string_pool = strings.as_ref().unwrap();
        let string = &string_pool[index];

        return string.to_owned();
    } else {
        return reader.read_string(None);
    }
}

/// Reads a vdf entry map to JSON.
pub fn read_entry_map(
    reader: &mut Reader,
    magic: Option<u32>,
    strings: &Option<&mut Vec<String>>,
) -> Map<String, Value> {
    let mut props = Map::new();
    let mut field_type = reader.read_uint8(true);

    while field_type != 0x08 {
        let key = read_vdf_string(reader, magic, strings);
        let value = read_entry_field(reader, field_type, magic, strings);

        props.insert(key, value);

        field_type = reader.read_uint8(true);
    }

    return props;
}

/// Reads a vdf entry field to JSON.
pub fn read_entry_field(
    reader: &mut Reader,
    field_type: u8,
    magic: Option<u32>,
    strings: &Option<&mut Vec<String>>,
) -> Value {
    match field_type {
        0x00 => {
            //? map
            return Value::Object(read_entry_map(reader, magic, strings));
        }
        0x01 => {
            //? string
            let value = reader.read_string(None);
            return Value::String(value);
        }
        0x02 => {
            //? number
            let value = reader.read_uint32(true);
            return Value::Number(value.into());
        }
        _ => {
            panic!("Unexpected field type {}!", field_type);
        }
    }
}
