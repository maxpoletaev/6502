use super::types::*;
use crate::mem::Memory;
use std::cell::RefCell;
use std::rc::Rc;

type MemRange = (Word, Word);

struct MappedMemory {
    range: MemRange,
    device: Rc<RefCell<dyn Memory>>,
}

pub struct Bus {
    devices: Vec<MappedMemory>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            devices: Vec::new(),
        }
    }

    pub fn plug_in(
        &mut self,
        range: MemRange,
        device: Rc<RefCell<dyn Memory>>,
    ) -> Result<(), String> {
        for existing_device in &self.devices {
            if existing_device.range.0 >= range.1 || existing_device.range.1 >= range.0 {
                return Err(format!(
                    "devices overlap: {:?} and {:?}",
                    existing_device.range, range
                ));
            }
        }

        self.devices.push(MappedMemory { range, device });
        Ok(())
    }
}

impl Memory for Bus {
    fn read(&self, addr: Word) -> Byte {
        for mapped_device in &self.devices {
            if mapped_device.range.0 <= addr && addr <= mapped_device.range.1 {
                let device = mapped_device.device.borrow();
                return device.read(addr - mapped_device.range.0);
            }
        }
        0
    }

    fn write(&mut self, addr: Word, data: Byte) {
        for mapped_device in &mut self.devices {
            if mapped_device.range.0 <= addr && addr <= mapped_device.range.1 {
                let mut device = mapped_device.device.borrow_mut();
                device.write(addr - mapped_device.range.0, data);
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    struct TestDevice {
        addr: Word,
        data: Byte,
    }

    impl Memory for TestDevice {
        fn read(&self, addr: Word) -> Byte {
            if addr == self.addr {
                self.data
            } else {
                0
            }
        }

        fn write(&mut self, addr: Word, data: Byte) {
            if addr == self.addr {
                self.data = data;
            }
        }
    }

    #[test]
    fn read() {
        let mut bus = Bus::new();
        let device = Rc::new(RefCell::new(TestDevice {
            addr: 0x1234,
            data: 0xAB,
        }));

        bus.plug_in((0x0000, 0xFFFF), device.clone()).unwrap();
        let val = bus.read(0x1234);
        assert_eq!(val, 0xAB)
    }

    #[test]
    fn read_offset() {
        let mut bus = Bus::new();
        let device = Rc::new(RefCell::new(TestDevice {
            addr: 0x0000,
            data: 0xAB,
        }));

        bus.plug_in((0x0001, 0xFFFF), device.clone()).unwrap();
        let val = bus.read(0x0001);
        assert_eq!(val, 0xAB)
    }

    #[test]
    fn write() {
        let device = Rc::new(RefCell::new(TestDevice {
            addr: 0xAAAA,
            data: 0x00,
        }));

        let mut bus = Bus::new();
        bus.plug_in((0x0000, 0xFFFF), device.clone()).unwrap();
        bus.write(0xAAAA, 0xFF);

        let device = device.borrow();
        assert_eq!(device.addr, 0xAAAA);
        assert_eq!(device.data, 0xFF);
    }

    #[test]
    fn write_offset() {
        let device = Rc::new(RefCell::new(TestDevice {
            addr: 0x0000,
            data: 0x00,
        }));

        let mut bus = Bus::new();
        bus.plug_in((0x0001, 0xFFFF), device.clone()).unwrap();
        bus.write(0x0001, 0xFF);

        let device = device.borrow();
        assert_eq!(device.addr, 0x0000);
        assert_eq!(device.data, 0xFF);
    }

    #[test]
    fn conflict() {
        let device = Rc::new(RefCell::new(TestDevice {
            addr: 0xAAAA,
            data: 0x00,
        }));

        let mut bus = Bus::new();
        bus.plug_in((0x0000, 0x1000), device.clone()).unwrap();
        bus.plug_in((0x0900, 0x2000), device.clone()).unwrap_err();
    }

    #[test]
    fn no_conflict() {
        let device = Rc::new(RefCell::new(TestDevice {
            addr: 0xAAAA,
            data: 0x00,
        }));

        let mut bus = Bus::new();
        bus.plug_in((0x0000, 0x1000), device.clone()).unwrap();
        bus.plug_in((0x1001, 0x2000), device.clone()).unwrap();
    }
}
