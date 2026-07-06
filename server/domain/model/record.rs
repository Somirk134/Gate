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

impl Display for RecordType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::A => "A",
            Self::Aaaa => "AAAA",
            Self::Cname => "CNAME",
            Self::Txt => "TXT",
            Self::MxReserved => "MX_RESERVED",
            Self::SrvReserved => "SRV_RESERVED",
        })
    }
}
