use memory::Memory;
use std::fs::File;
use std::io::prelude::*;

mod cartridge_metadata;
use cartridge_metadata::CartridgeMetadata;
pub use cartridge_metadata::Mirroring;

mod mapper;
use mapper::*;
use mapper0::Mapper0;

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

        let prg_data = bytes
            .by_ref()
            .take(0x4000 * (meta.n_prg_banks as usize))
            .map(|x| x.unwrap())
            .collect::<Vec<u8>>();
        let chr_data = bytes
            .by_ref()
            .take(0x2000 * (meta.n_chr_banks as usize))
            .map(|x| x.unwrap())
            .collect::<Vec<u8>>();

        let mapper = Box::from(match meta.mapper_num {
            0 => Mapper0::new(meta.n_prg_banks, meta.n_chr_banks, prg_data, chr_data),
            _ => return Err("unsupported mapper"),
        });

        Ok(Self { meta, mapper })
    }

    pub fn get_nametable_mirroring(&self) -> Mirroring {
        self.mapper
            .get_nametable_mirroring()
            .unwrap_or(self.meta.hardwired_mirroring)
    }
}

impl Memory for Cartridge {
    fn read(&mut self, addr: u16) -> u8 {
        self.mapper.read(addr)
    }

    fn peek(&self, addr: u16) -> u8 {
        self.mapper.peek(addr)
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.mapper.write(addr, data);
    }
}
