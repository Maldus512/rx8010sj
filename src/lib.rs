use datetime::*;
use embedded_hal::i2c::I2c;

const DEFAULT_ADDRESS: u8 = 0x64 >> 1;
const REGISTER_CONTROL: u8 = 0x1F;
const REGISTER_SEC: u8 = 0x10;

const BIT_REGISTER_CONTROL_STOP: u8 = 0x40;

/// RX-8010-SJ
/// Real-Time Clock (RTC) Module with I2C-Bus Interface
/// rust no_std driver (utilizes the embedded_hal i2c interface)
pub struct Rx8010sj<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> Rx8010sj<I2C>
where
    I2C: I2c<Error = E>,
{
    /// New driver instance, assumes that there is no i2c mux
    /// sitting between the RTC and the host.
    pub fn new(i2c: I2C) -> Self {
        Rx8010sj {
            i2c,
            address: DEFAULT_ADDRESS,
        }
    }

    pub fn with_address(self: Self, address: u8) -> Self {
        Rx8010sj { address, ..self }
    }

    pub fn is_stopped(self: &mut Self) -> Result<bool, E> {
        let control_register = self.read_register(REGISTER_CONTROL)?;
        Ok((control_register & BIT_REGISTER_CONTROL_STOP) > 0)
    }

    pub fn set_stopped(self: &mut Self, stopped: bool) -> Result<(), E> {
        let control_register = self.read_register(REGISTER_CONTROL)?;
        self.write_register(
            REGISTER_CONTROL,
            if stopped {
                control_register | BIT_REGISTER_CONTROL_STOP
            } else {
                control_register & (!BIT_REGISTER_CONTROL_STOP)
            },
        )?;
        Ok(())
    }

    pub fn get_time(self: &mut Self) -> Result<LocalDateTime, E> {
        let time_registers = self.read_registers::<7>(REGISTER_SEC)?;

        let sec = bcd2bin(time_registers[0]);
        let min = bcd2bin(time_registers[1]);
        let hour = bcd2bin(time_registers[2]);
        let wday = bcd2bin(time_registers[3]);
        let day = bcd2bin(time_registers[4]);
        let month = bcd2bin(time_registers[5]);
        let year = bcd2bin(time_registers[6]);

        let date = LocalDate::ymd(
            year as i64,
            Month::from_zero(month as i8).unwrap_or(Month::January),
            day as i8,
        )
        .unwrap_or(LocalDate::yd(1970, 0).unwrap());

        let time = LocalTime::hms(hour as i8, min as i8, sec as i8)
            .unwrap_or(LocalTime::hm(0, 0).unwrap());

        Ok(LocalDateTime::new(date, time))
    }

    pub fn set_time(self: &mut Self, date_time: LocalDateTime) -> Result<(), E> {
        let date = date_time.date();
        let time = date_time.time();

        let time_registers: [u8;7] =[ bin2bcd(time.second() as u8),
         bin2bcd(time.minute() as u8)
         ,bin2bcd(time.hour() as u8)
         ,bin2bcd(match date.weekday() {Weekday::Sunday => 0, Weekday::Monday => 1, Weekday::Tuesday => 2, Weekday::Wednesday => 3, Weekday::Thursday => 4, Weekday::Friday => 5, Weekday::Saturday=>6} )
         ,bin2bcd(date.day() as u8)
         ,bin2bcd(match date.month() {
            Month::January => 0,
            Month::February => 1,
            Month::March => 2,
            Month::April => 3,
            Month::May => 4,
            Month::June => 5,
            Month::July => 6,
            Month::August => 7,
            Month::September => 8,
            Month::October => 9,
            Month::November => 10,
            Month::December => 11,
        })
         ,bin2bcd((date.year() - 1900) as u8)];

        self.write_registers(REGISTER_SEC,&time_registers)?;

        Ok(())
    }


    fn write_register(self: &mut Self, reg: u8, data: u8) -> Result<(), E> {
        self.i2c.write(self.address, &[reg, data])
    }

    fn write_registers<const N: usize>(self: &mut Self, reg: u8, data: &[u8; N]) -> Result<(), E> {
        for i in 0..N {
            self.write_register(reg+(i as u8), data[i])?;
        }
        Ok(())

        /*
        let buffer: [u8; N+1] = [0;N+1];
        buffer[0] = reg;
        for i in 1..N+1 {
            buffer[i] = data[i-1];
        }
        self.i2c.write(self.address, buffer)
        */
    }

    fn read_register(self: &mut Self, reg: u8) -> Result<u8, E> {
        self.read_registers::<1>(reg).map(|regs| regs[0])
    }

    fn read_registers<const N: usize>(self: &mut Self, reg: u8) -> Result<[u8; N], E> {
        let mut buf: [u8; N] = [0; N];
        self.i2c.write_read(self.address, &[reg], &mut buf)?;
        Ok(buf)
    }
}

fn bcd2bin(bcd: u8) -> u8 {
    ((bcd >> 4) & 0xF) * 10 + ((bcd) & 0xF)
}

fn bin2bcd(bin: u8) -> u8 {
    (((bin) / 10) << 4) | ((bin) % 10)
}
