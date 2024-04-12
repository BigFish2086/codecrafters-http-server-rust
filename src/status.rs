use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatusCode(u16);

pub struct InvalidStatusCode {
    _priv: (),
}

impl StatusCode {
    #[allow(dead_code)]
    #[inline]
    pub fn from_u16(num: u16) -> Result<StatusCode, InvalidStatusCode> {
        if !(100..1000).contains(&num) {
            Err(InvalidStatusCode::new())
        } else {
            Ok(StatusCode(num))
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn as_u16(&self) -> u16 {
        self.0
    }

    pub fn reason(&self) -> Option<&'static str> {
        reason(self.0)
    }
}

impl fmt::Debug for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.0,
            self.reason().unwrap_or("<unknown status code>")
        )
    }
}

impl InvalidStatusCode {
    fn new() -> InvalidStatusCode {
        InvalidStatusCode { _priv: () }
    }
}

impl fmt::Debug for InvalidStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InvalidStatusCode").finish()
    }
}

impl fmt::Display for InvalidStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid status code")
    }
}

impl Error for InvalidStatusCode {}

macro_rules! status_codes {
    ( $( ($num:expr, $konst:ident, $phrase:expr);)+) => {
        impl StatusCode {
            $( pub const $konst: StatusCode = StatusCode($num); )+
        }
        fn reason(num: u16) -> Option<&'static str> {
            match num {
                $( $num => Some($phrase),)+
                _ => None
            }
        }
    }
}

status_codes! {
    (200, OK, "OK");
    (201, CREATED, "Created");
    (404, NOT_FOUND, "Not Found");
    (500, INTERNAL_SERVER_ERROR, "Internal Server Error");
}
