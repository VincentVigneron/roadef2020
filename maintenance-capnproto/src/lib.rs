extern crate capnp;
extern crate maintenance;
extern crate uuid;

use maintenance::common::Maintenance;

pub mod maintenance_capnp {
    include!(concat!(env!("OUT_DIR"), "/maintenance_capnp.rs"));
}

pub struct MaintenanceSummaryBuilder {
    message: ::capnp::message::Builder<::capnp::message::HeapAllocator>,
}

pub struct MaintenanceSummaryReader {
    message: ::capnp::message::Reader<::capnp::serialize::OwnedSegments>,
}

pub struct MaintenanceSummary<'a> {
    reader: crate::maintenance_capnp::maintenance::Reader<'a>,
}

impl<'a> MaintenanceSummary<'a> {
    pub fn ndays(&self) -> u32 {
        self.reader.get_ndays()
    }

    pub fn ninterventions(&self) -> u32 {
        self.reader.get_ninterventions()
    }

    pub fn nresources(&self) -> u32 {
        self.reader.get_nresources()
    }

    pub fn nscenarios(&self) -> u32 {
        self.reader.get_nscenarios()
    }
}

pub mod capnp_uuid {
    pub fn encode(uuid: &uuid::Uuid) -> ::capnp::Result<Vec<u8>> {
        let mut message = ::capnp::message::Builder::new_default();
        {
            let message = message.init_root::<crate::maintenance_capnp::maintenance_id::Builder>();
            let mut id = message.init_bytes(16);
            for (i, &bytes) in uuid.as_bytes().iter().enumerate() {
                id.set(i as u32, bytes);
            }
        }
        let mut data = Vec::new();
        ::capnp::serialize::write_message(&mut data, &message)?;
        Ok(data)
    }

    pub fn decode(data: &[u8]) -> ::capnp::Result<uuid::Uuid> {
        let mut reader = std::io::BufReader::new(data);
        let message =
            ::capnp::serialize::read_message(&mut reader, ::capnp::message::ReaderOptions::new())?;
        let uuid = message.get_root::<crate::maintenance_capnp::maintenance_id::Reader>()?;
        let mut bytes = uuid.get_bytes()?.iter();
        let uuid = [
            bytes.next().unwrap(),
            bytes.next().unwrap(),
            bytes.next().unwrap(),
            bytes.next().unwrap(),
            bytes.next().unwrap(),
            bytes.next().unwrap(),
            bytes.next().unwrap(),
            bytes.next().unwrap(),
            bytes.next().unwrap(),
            bytes.next().unwrap(),
            bytes.next().unwrap(),
            bytes.next().unwrap(),
            bytes.next().unwrap(),
            bytes.next().unwrap(),
            bytes.next().unwrap(),
            bytes.next().unwrap(),
        ];
        Ok(uuid::Uuid::from_bytes(uuid))
    }
}

impl MaintenanceSummaryBuilder {
    pub fn from_maintenance(maintenance: &Maintenance) -> Self {
        let mut message = ::capnp::message::Builder::new_default();
        {
            let mut m = message.init_root::<crate::maintenance_capnp::maintenance::Builder>();
            m.set_ndays(maintenance.ndays() as u32);
            m.set_ninterventions(maintenance.ninterventions() as u32);
            m.set_nresources(maintenance.nresources() as u32);
            m.set_nscenarios(maintenance.nscenarios() as u32);
        }
        MaintenanceSummaryBuilder { message: message }
    }

    // for debug purpose
    pub fn reader<'a>(&'a self) -> MaintenanceSummary<'a> {
        MaintenanceSummary {
            reader: self
                .message
                .get_root_as_reader::<crate::maintenance_capnp::maintenance::Reader<'a>>()
                .expect("message"),
        }
    }

    pub fn bytes(&self) -> ::capnp::Result<Vec<u8>> {
        let mut data = Vec::new();
        ::capnp::serialize::write_message(&mut data, &self.message)?;
        Ok(data)
    }
}

impl MaintenanceSummaryReader {
    pub fn from_bytes(data: &[u8]) -> ::capnp::Result<Self> {
        let mut reader = std::io::BufReader::new(data);
        let message =
            ::capnp::serialize::read_message(&mut reader, ::capnp::message::ReaderOptions::new())?;
        Ok(MaintenanceSummaryReader { message: message })
    }

    pub fn reader<'a>(&'a self) -> MaintenanceSummary<'a> {
        MaintenanceSummary {
            reader: self
                .message
                .get_root::<crate::maintenance_capnp::maintenance::Reader>()
                .expect("message"),
        }
    }
}
