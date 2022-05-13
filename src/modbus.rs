use core::convert::Infallible;
use core::fmt::Debug;
use core::task::Poll;
use defmt::Format;
use embedded_hal::serial::{Read, Write};
use fixedvec::{ErrorKind as FixedVecError, FixedVec};
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

pub struct ModbusSerial<'a, SerialTx, SerialRx>
where
    SerialTx: Write<u8>,
    SerialRx: Read<u8>,
{
    tx: SerialTx,
    rx: SerialRx,
    status: ModbusSerialStatus,
    request: ModbusRequest,
    request_bytes: FixedVec<'a, u8>,
    request_bytes_index: usize,
    response_bytes: FixedVec<'a, u8>,
    response_bytes_length: Option<u8>,
    response_ready: bool,
}

pub enum ModbusSerialError<SerialTx, SerialRx>
where
    SerialTx: Write<u8>,
    SerialRx: Read<u8>,
{
    SerialTx(SerialTx::Error),
    SerialRx(SerialRx::Error),
    FixedVec(FixedVecError),
    Modbus(ModbusError),
    Unexpected,
}

impl<'a, SerialTx, SerialRx> ModbusSerial<'a, SerialTx, SerialRx>
where
    SerialTx: Write<u8>,
    SerialRx: Read<u8>,
{
    pub fn new(
        tx: SerialTx,
        rx: SerialRx,
        unit_id: u8,
        request_bytes_space: &'a mut [u8],
        response_bytes_space: &'a mut [u8],
    ) -> Self {
        Self {
            tx,
            rx,
            status: ModbusSerialStatus::Idle,
            request: ModbusRequest::new(unit_id, ModbusProto::Rtu),
            request_bytes: FixedVec::new(request_bytes_space),
            request_bytes_index: 0,
            response_bytes: FixedVec::new(response_bytes_space),
            response_bytes_length: None,
            response_ready: false,
        }
    }

    pub fn get_coils(&mut self, reg: u16, count: u16) -> Result<(), ModbusError> {
        self.request
            .generate_get_coils(reg, count, &mut self.request_bytes)?;
        self.status = ModbusSerialStatus::Writing;
        Ok(())
    }

    pub fn get_discretes(&mut self, reg: u16, count: u16) -> Result<(), ModbusError> {
        self.request
            .generate_get_discretes(reg, count, &mut self.request_bytes)?;
        self.status = ModbusSerialStatus::Writing;
        Ok(())
    }

    pub fn get_inputs(&mut self, reg: u16, count: u16) -> Result<(), ModbusError> {
        self.request
            .generate_get_inputs(reg, count, &mut self.request_bytes)?;
        self.status = ModbusSerialStatus::Writing;
        Ok(())
    }

    pub fn set_coil(&mut self, reg: u16, value: bool) -> Result<(), ModbusError> {
        self.request
            .generate_set_coil(reg, value, &mut self.request_bytes)?;
        self.status = ModbusSerialStatus::Writing;
        Ok(())
    }

    pub fn set_holding(&mut self, reg: u16, value: u16) -> Result<(), ModbusError> {
        self.request
            .generate_set_holding(reg, value, &mut self.request_bytes)?;
        self.status = ModbusSerialStatus::Writing;
        Ok(())
    }

    pub fn set_holdings_bulk(&mut self, reg: u16, values: &[u16]) -> Result<(), ModbusError> {
        self.request
            .generate_set_holdings_bulk(reg, values, &mut self.request_bytes)?;
        self.status = ModbusSerialStatus::Writing;
        Ok(())
    }

    pub fn set_coils_bulk(&mut self, reg: u16, values: &[bool]) -> Result<(), ModbusError> {
        self.request
            .generate_set_coils_bulk(reg, values, &mut self.request_bytes)?;
        self.status = ModbusSerialStatus::Writing;
        Ok(())
    }

    pub fn parse_ok(&mut self) -> Result<(), ModbusError> {
        self.response_ready = false;

        self.request.parse_ok(self.response_bytes.as_slice())
    }

    pub fn parse_u16<V: VectorTrait<u16>>(&mut self, result: &mut V) -> Result<(), ModbusError> {
        self.response_ready = false;

        self.request
            .parse_u16(self.response_bytes.as_slice(), result)
    }

    pub fn parse_bool<V: VectorTrait<bool>>(&mut self, result: &mut V) -> Result<(), ModbusError> {
        self.response_ready = false;

        self.request
            .parse_bool(self.response_bytes.as_slice(), result)
    }

    pub fn poll(&mut self) -> Poll<Result<bool, ModbusSerialError<SerialTx, SerialRx>>> {
        match self.status {
            ModbusSerialStatus::Idle => Poll::Ready(Ok(self.response_ready)),
            ModbusSerialStatus::Writing => {
                if let Some(next_byte) = self.request_bytes.get(self.request_bytes_index) {
                    self.request_bytes_index += 1;

                    match self.tx.write(*next_byte) {
                        Ok(()) => Poll::Pending,
                        Err(nb::Error::WouldBlock) => Poll::Pending,
                        Err(nb::Error::Other(err)) => {
                            Poll::Ready(Err(ModbusSerialError::SerialTx(err)))
                        }
                    }
                } else {
                    match self.tx.flush() {
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

                match self.rx.read() {
                    Ok(next_byte) => {
                        self.response_bytes
                            .push(next_byte)
                            .map_err(|err| ModbusSerialError::FixedVec(err))?;

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
