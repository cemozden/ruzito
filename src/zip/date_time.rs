use std::{fmt::Display, write};

const DAY_MASK: u16   = 0b11111;
const MONTH_MASK: u16 = 0b1111;
const YEAR_MASK: u16  = 0b1111111;

const HOUR_MASK: u16 = 0b11111;
const MINUTE_MASK: u16 = 0b111111;
const SECOND_MASK: u16 = 0b11111;

const MS_DOS_YEAR_START_OFFSET: u16 = 1980;

#[derive(Debug, PartialEq, Eq)]
pub struct ZipDateTime {
    day: u8,
    month: u8,
    year: u16,
    hour: u8,
    minute: u8,
    second: u8
}

impl ToOwned for ZipDateTime {
    type Owned = ZipDateTime;

    fn to_owned(&self) -> Self::Owned {
        ZipDateTime {
            day: self.day,
            month: self.month,
            year: self.year,
            hour: self.hour,
            minute: self.minute,
            second: self.second
        }
    }
}

impl ZipDateTime {

    pub fn new(day: u8, month: u8, year: u16, hour: u8, minute: u8, second: u8) -> Self {
        ZipDateTime {
            day,
            month,
            year,
            hour,
            minute,
            second
        }
    }

    pub fn from_addr(date_addr: u16, time_addr: u16) -> Self {
        let day = (date_addr & DAY_MASK) as u8;
        let month = (date_addr >> 5 & MONTH_MASK) as u8;
        let year = (date_addr >> 9 & YEAR_MASK) + MS_DOS_YEAR_START_OFFSET;

        let hour = (time_addr >> 11 & HOUR_MASK) as u8;
        let minute = (time_addr >> 5 & MINUTE_MASK) as u8;
        let second = ((time_addr & SECOND_MASK) * 2) as u8;

        ZipDateTime {
            day,
            month,
            year,
            hour,
            minute,
            second
        }
    }

    pub fn to_addr(self,  date_addr: &mut u16, time_addr: &mut u16) {
        let month = (self.month << 5) as u16;
        let year = (self.year - MS_DOS_YEAR_START_OFFSET) << 9;

        let hour = (self.hour as u16) << 11;
        let minute = (self.minute as u16) << 5;
        let second = (self.second / 2) as u16;

        *date_addr = year | month | (self.day as u16);
        *time_addr = hour | minute | second;
    }
}

impl Display for ZipDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}/{:02}/{} {:02}:{:02}:{:02}", self.month, self.day, self.year, self.hour, self.minute, self.second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date() {
        let date = ZipDateTime::from_addr(0x5162, 0x0);
        assert_eq!(date.day, 2);
        assert_eq!(date.month, 11);
        assert_eq!(date.year, 2020);
    }

    #[test]
    fn test_time() {
        let time = ZipDateTime::from_addr(0x0, 0xA9F4);

        assert_eq!(time.hour, 21);
        assert_eq!(time.minute, 15);
        assert_eq!(time.second, 40);
        println!("{:?}", time);
    }

    #[test]
    fn test_to_addr() {
        let time = ZipDateTime::new(1,3,2021,20,41, 56);

        let mut date_addr = 0;
        let mut time_addr = 0;

        time.to_addr(&mut date_addr, &mut time_addr);

        assert_eq!(date_addr, 0x5261);
        assert_eq!(time_addr, 0xA53C);
    }
}