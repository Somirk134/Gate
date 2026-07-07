use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RecordType {
    A,
    Aaaa,
    Cname,
    Txt,
    MxReserved,
    SrvReserved,
}

impl RecordType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::Aaaa => "AAAA",
            Self::Cname => "CNAME",
            Self::Txt => "TXT",
            Self::MxReserved => "MX_RESERVED",
            Self::SrvReserved => "SRV_RESERVED",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "A" => Some(Self::A),
            "AAAA" => Some(Self::Aaaa),
            "CNAME" => Some(Self::Cname),
            "TXT" => Some(Self::Txt),
            "MX_RESERVED" => Some(Self::MxReserved),
            "SRV_RESERVED" => Some(Self::SrvReserved),
            _ => None,
        }
    }
}

impl Display for RecordType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
