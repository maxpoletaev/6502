use super::types::*;
use std::collections::HashMap;

pub trait Device {
    fn read(&self, addr: Word) -> Byte;
    fn write(&mut self, addr: Word, data: Byte);
}

type Key = (Word, Word);

pub struct Bus {
    addr: Word,
    devices: HashMap<Key, Box<dyn Device>>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            addr: 0,
            devices: HashMap::new(),
        }
    }

    pub fn set_addr(&mut self, addr: Word) {
        self.addr = addr;
    }

    pub fn read(&self) -> Byte {
        match self.find(self.addr) {
            Some(key) => {
                let device = self.devices.get(&key).unwrap();
                device.read(self.addr - key.0)
            }
            None => 0,
        }
    }

    pub fn read_addr(&mut self, addr: Word) -> Byte {
        self.set_addr(addr);
        self.read()
    }

    pub fn write(&mut self, data: Byte) {
        match self.find(self.addr) {
            Some(key) => {
                let device = self.devices.get_mut(&key).unwrap();
                device.write(self.addr - key.0, data);
            }
            None => (),
        }
    }

    pub fn write_addr(&mut self, addr: Word, data: Byte) {
        self.set_addr(addr);
        self.write(data);
    }

    pub fn plug_in(&mut self, key: Key, device: Box<dyn Device>) -> Result<(), String> {
        for r in self.devices.keys() {
            if r.0 >= key.1 || r.1 <= key.0 {
                return Err(format!("devices overlap: {:?} and {:?}", r, key));
            }
        }
        self.devices.insert(key, device);
        Ok(())
    }

    pub fn find(&self, addr: Word) -> Option<Key> {
        for key in self.devices.keys() {
            if addr >= key.0 && addr <= key.1 {
                return Some(key.clone());
            }
        }
        None
    }
}
