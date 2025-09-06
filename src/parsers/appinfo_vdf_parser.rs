/*
 * This file is part of Steam-Art-Manager which is licensed under GNU Lesser General Public License v2.1
 * See file LICENSE or go to https://www.gnu.org/licenses/old-licenses/lgpl-2.1.en.html for full license details
 */

use std::i64;
use std::io::Read;
use std::{fs, path::PathBuf};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde_json::{Map, Value};

use super::reader::Reader;
use super::vdf_reader::read_entry_map;

/// Opens the appinfo.vdf file and returns the values as JSON.
pub fn open_appinfo_vdf(path: &PathBuf) -> Map<String, Value> {
    let mut file = fs::File::open(path).expect("Path should have existed.");
    let metadata = fs::metadata(path).expect("Unable to read metadata.");

    let mut buffer = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut buffer).expect("Buffer overflow.");

    let mut reader = Reader::new(&buffer);

    return read(&mut reader);
}

/// Reads the appinfo.vdf file and returns the values as JSON.
fn read(reader: &mut Reader) -> Map<String, Value> {
    let magic = reader.read_uint32(true);
    let _universe = reader.read_uint32(true); //always 1

    let entries: Vec<Value>;

    if magic == 0x07564429 {
        let string_table_offset = reader.read_int64(true);
        let data_offset = reader.get_offset();

        reader.seek(
            string_table_offset
                .try_into()
                .expect("String table offset couldn't be converted to usize"),
            0,
        );

        let string_count = reader.read_uint32(true) as usize;
        let mut strings = Vec::with_capacity(string_count);

        for _ in 0..string_count {
            strings.push(reader.read_string(None));
        }

        reader.seek(data_offset, 0);

        entries = read_app_sections(
            reader,
            Some(string_table_offset),
            Some(magic),
            &Some(&mut strings),
        );
    } else if magic == 0x07564428 {
        entries = read_app_sections(reader, None, None, &None);
    } else {
        panic!("Magic header is unknown. Expected 0x07564428 or 0x07564429 but got {magic}");
    }

    let mut res: Map<String, Value> = Map::new();
    res.insert(String::from("entries"), Value::Array(entries));

    return res;
}

struct AppInfoChunk {
    pub offset: usize,
    pub length: usize,
}

/// Reads the appinfo.vdf app sections to a JSON array.
fn read_app_sections(
    reader: &mut Reader,
    string_table_offset: Option<i64>,
    magic: Option<u32>,
    strings: &Option<&mut Vec<String>>,
) -> Vec<Value> {
    let mut id = reader.read_uint32(true);
    let eof = string_table_offset.unwrap_or(i64::MAX) as usize - 4;

    let mut chunks = Vec::new();

    while id != 0 && reader.get_offset() < eof {
        let chunk_size = reader.read_uint32(true);
        let offset = reader.get_offset();
        let chunk_length: usize = chunk_size.try_into().unwrap();

        chunks.push(AppInfoChunk {
            offset,
            length: chunk_length,
        });

        reader.seek(offset + chunk_length, 0);
        id = reader.read_uint32(true);
    }

    let entries: Vec<Value> = chunks
        .par_iter()
        .filter_map(|chunk| {
            let mut chunk_reader = reader.slice(chunk.offset, chunk.length);
            chunk_reader.seek(60, 1);

            let mut entry: Map<String, Value> = read_entry_map(&mut chunk_reader, magic, strings);

            if entry.contains_key("appinfo") {
                let appinfo_val: &Value = entry
                    .get("appinfo")
                    .expect("Should have been able to get \"appinfo\".");
                let appinfo = appinfo_val
                    .as_object()
                    .expect("appinfo should have been an object.");

                entry = appinfo.clone();
            }

            return Some(Value::Object(entry));
        })
        .collect();

    return entries;
}
