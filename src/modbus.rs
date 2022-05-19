use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use embedded_hal::serial::{Read, Write};
use heapless::Vec;
use nb;
use rmodbus::{
    client::ModbusRequest, guess_response_frame_len, ErrorKind as ModbusError, ModbusProto,
    VectorTrait,
};

#[derive(Clone, Copy, Debug, Format, PartialEq)]
enum ModbusSerialStatus {
    Idle,
    Writing,
    Reading,
}

pub struct ModbusSerial<Serial>
where
    Serial: Write<u8> + Read<u8>,
{
    serial: Serial,
    status: ModbusSerialStatus,
    request: ModbusRequest,
    request_bytes: Vec<u8, 64>,
    request_bytes_index: usize,
    response_bytes: Vec<u8, 64>,
    response_bytes_length: Option<u8>,
    response_ready: bool,
}

#[derive(Debug)]
pub enum ModbusSerialError<SerialTxError: Debug, SerialRxError: Debug> {
    SerialTx(SerialTxError),
    SerialRx(SerialRxError),
    Vec,
    Modbus(ModbusError),
    Unexpected,
}

pub type ModbusSerialErrorAlias<Serial> =
    ModbusSerialError<<Serial as Write<u8>>::Error, <Serial as Read<u8>>::Error>;

impl<Serial> ModbusSerial<Serial>
where
    Serial: Write<u8> + Read<u8>,
    <Serial as Write<u8>>::Error: Debug,
    <Serial as Read<u8>>::Error: Debug,
{
    pub fn new(serial: Serial, unit_id: u8) -> Self {
        Self {
            serial,
            status: ModbusSerialStatus::Idle,
            request: ModbusRequest::new(unit_id, ModbusProto::Rtu),
            request_bytes: Vec::new(),
            request_bytes_index: 0,
            response_bytes: Vec::new(),
            response_bytes_length: None,
            response_ready: false,
        }
    }

    pub fn get_coils(
        &mut self,
        reg: u16,
        count: u16,
    ) -> Result<(), ModbusSerialErrorAlias<Serial>> {
        self.request
            .generate_get_coils(reg, count, &mut self.request_bytes)
            .map_err(|err| ModbusSerialError::Modbus(err))?;
        self.status = ModbusSerialStatus::Writing;
        Ok(())
    }

    pub fn get_discretes(
        &mut self,
        reg: u16,
        count: u16,
    ) -> Result<(), ModbusSerialErrorAlias<Serial>> {
        self.request
            .generate_get_discretes(reg, count, &mut self.request_bytes)
            .map_err(|err| ModbusSerialError::Modbus(err))?;
        self.status = ModbusSerialStatus::Writing;
        Ok(())
    }

    pub fn get_inputs(
        &mut self,
        reg: u16,
        count: u16,
    ) -> Result<(), ModbusSerialErrorAlias<Serial>> {
        self.request
            .generate_get_inputs(reg, count, &mut self.request_bytes)
            .map_err(|err| ModbusSerialError::Modbus(err))?;
        self.status = ModbusSerialStatus::Writing;
        Ok(())
    }

    pub fn set_coil(
        &mut self,
        reg: u16,
        value: bool,
    ) -> Result<(), ModbusSerialErrorAlias<Serial>> {
        self.request
            .generate_set_coil(reg, value, &mut self.request_bytes)
            .map_err(|err| ModbusSerialError::Modbus(err))?;
        self.status = ModbusSerialStatus::Writing;
        Ok(())
    }

    pub fn set_holding(
        &mut self,
        reg: u16,
        value: u16,
    ) -> Result<(), ModbusSerialErrorAlias<Serial>> {
        self.request
            .generate_set_holding(reg, value, &mut self.request_bytes)
            .map_err(|err| ModbusSerialError::Modbus(err))?;
        self.status = ModbusSerialStatus::Writing;
        Ok(())
    }

    pub fn set_holdings_bulk(
        &mut self,
        reg: u16,
        values: &[u16],
    ) -> Result<(), ModbusSerialErrorAlias<Serial>> {
        self.request
            .generate_set_holdings_bulk(reg, values, &mut self.request_bytes)
            .map_err(|err| ModbusSerialError::Modbus(err))?;
        self.status = ModbusSerialStatus::Writing;
        Ok(())
    }

    pub fn set_coils_bulk(
        &mut self,
        reg: u16,
        values: &[bool],
    ) -> Result<(), ModbusSerialErrorAlias<Serial>> {
        self.request
            .generate_set_coils_bulk(reg, values, &mut self.request_bytes)
            .map_err(|err| ModbusSerialError::Modbus(err))?;
        self.status = ModbusSerialStatus::Writing;
        Ok(())
    }

    pub fn parse_ok(&mut self) -> Result<(), ModbusSerialErrorAlias<Serial>> {
        self.response_ready = false;

        self.request
            .parse_ok(self.response_bytes.as_slice())
            .map_err(|err| ModbusSerialError::Modbus(err))
    }

    pub fn parse_u16<V: VectorTrait<u16>>(
        &mut self,
        result: &mut V,
    ) -> Result<(), ModbusSerialErrorAlias<Serial>> {
        self.response_ready = false;

        self.request
            .parse_u16(self.response_bytes.as_slice(), result)
            .map_err(|err| ModbusSerialError::Modbus(err))
    }

    pub fn parse_bool<V: VectorTrait<bool>>(
        &mut self,
        result: &mut V,
    ) -> Result<(), ModbusSerialErrorAlias<Serial>> {
        self.response_ready = false;

        self.request
            .parse_bool(self.response_bytes.as_slice(), result)
            .map_err(|err| ModbusSerialError::Modbus(err))
    }

    pub fn poll(&mut self) -> Poll<Result<bool, ModbusSerialErrorAlias<Serial>>> {
        match self.status {
            ModbusSerialStatus::Idle => Poll::Ready(Ok(self.response_ready)),
            ModbusSerialStatus::Writing => {
                if let Some(next_byte) = self.request_bytes.get(self.request_bytes_index) {
                    self.request_bytes_index += 1;

                    match self.serial.write(*next_byte) {
                        Ok(()) => Poll::Pending,
                        Err(nb::Error::WouldBlock) => Poll::Pending,
                        Err(nb::Error::Other(err)) => {
                            Poll::Ready(Err(ModbusSerialError::SerialTx(err)))
                        }
                    }
                } else {
                    match self.serial.flush() {
                        Ok(()) => {
                            self.request_bytes_index = 0;
                            self.request_bytes.clear();

                            self.response_bytes_length = None;
                            self.response_bytes.clear();

                            self.status = ModbusSerialStatus::Reading;

                            Poll::Pending
                        }
                        Err(nb::Error::WouldBlock) => Poll::Pending,
                        Err(nb::Error::Other(err)) => {
                            Poll::Ready(Err(ModbusSerialError::SerialTx(err)))
                        }
                    }
                }
            }
            ModbusSerialStatus::Reading => {
                // if we've read enough, stop reading and return result
                if self.response_bytes_length.is_some()
                    && self.response_bytes.len() > (self.response_bytes_length.unwrap() as usize)
                {
                    self.status = ModbusSerialStatus::Idle;
                    self.response_ready = true;

                    return Poll::Ready(Ok(true));
                }

                match self.serial.read() {
                    Ok(next_byte) => {
                        self.response_bytes
                            .push(next_byte)
                            .map_err(|_err| ModbusSerialError::Vec)?;

                        if self.response_bytes_length.is_none() && self.response_bytes.len() > 6 {
                            let response_bytes_length = guess_response_frame_len(
                                self.response_bytes.as_slice(),
                                ModbusProto::Rtu,
                            )
                            .map_err(|err| ModbusSerialError::Modbus(err))?;
                            self.response_bytes_length = Some(response_bytes_length);
                        }

                        Poll::Pending
                    }
                    Err(nb::Error::WouldBlock) => Poll::Pending,
                    Err(nb::Error::Other(err)) => {
                        Poll::Ready(Err(ModbusSerialError::SerialRx(err)))
                    }
                }
            }
        }
    }
}
