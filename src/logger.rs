use log::{Log, Metadata, Record};
use crate::serial_println;

pub struct SerialLogger;

impl Log for SerialLogger
{
    fn enabled(&self, _metadata: &Metadata) -> bool
    {
        return true;
    }

    fn log(&self, record: &Record)
    {
        if self.enabled(record.metadata())
        {
            serial_println!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) { }
}

pub static SERIAL_LOGGER: SerialLogger = SerialLogger;