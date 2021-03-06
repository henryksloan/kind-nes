use memory::Memory;
use std::fs::File;
use std::io::prelude::*;

mod cartridge_metadata;
use cartridge_metadata::CartridgeMetadata;

pub struct Cartridge {
    meta: CartridgeMetadata,
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn from_file(file: File) -> Result<Self, &'static str> {
        let mut bytes = file.bytes();
        let header = match bytes.by_ref().take(0x10).collect() {
            Ok(header) => header,
            Err(_) => return Err("hit EOF while reading file header"),
        };
        let meta = CartridgeMetadata::from_header(header)?;

        let mapper = match meta.mapper_num {
            _ => return Err(format!("mapper {} is not supported", meta.mapper_num)),
        }

        Ok(Self { meta, mapper })
    }
}

trait Mapper: Memory {}
