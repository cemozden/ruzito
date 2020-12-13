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
}