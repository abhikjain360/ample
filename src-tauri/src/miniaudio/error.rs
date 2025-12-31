use thiserror::Error;

#[repr(i32)]
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error("miniaudio error: {0}")]
pub enum Error {
    #[error("Unknown error")]
    Error = -1,
    #[error("Invalid arguments")]
    InvalidArgs = -2,
    #[error("Invalid operation")]
    InvalidOperation = -3,
    #[error("Out of memory")]
    OutOfMemory = -4,
    #[error("Out of range")]
    OutOfRange = -5,
    #[error("Access denied")]
    AccessDenied = -6,
    #[error("Does not exist")]
    DoesNotExist = -7,
    #[error("Already exists")]
    AlreadyExists = -8,
    #[error("Too many open files")]
    TooManyOpenFiles = -9,
    #[error("Invalid file")]
    InvalidFile = -10,
    #[error("Too big")]
    TooBig = -11,
    #[error("Path too long")]
    PathTooLong = -12,
    #[error("Name too long")]
    NameTooLong = -13,
    #[error("Not a directory")]
    NotDirectory = -14,
    #[error("Is a directory")]
    IsDirectory = -15,
    #[error("Directory not empty")]
    DirectoryNotEmpty = -16,
    #[error("At end")]
    AtEnd = -17,
    #[error("No space")]
    NoSpace = -18,
    #[error("Busy")]
    Busy = -19,
    #[error("IO error")]
    IOError = -20,
    #[error("Interrupt")]
    Interrupt = -21,
    #[error("Unavailable")]
    Unavailable = -22,
    #[error("Already in use")]
    AlreadyInUse = -23,
    #[error("Bad address")]
    BadAddress = -24,
    #[error("Bad seek")]
    BadSeek = -25,
    #[error("Bad pipe")]
    BadPipe = -26,
    #[error("Deadlock")]
    Deadlock = -27,
    #[error("Too many links")]
    TooManyLinks = -28,
    #[error("Not implemented")]
    NotImplemented = -29,
    #[error("No message")]
    NoMessage = -30,
    #[error("Bad message")]
    BadMessage = -31,
    #[error("No data available")]
    NoDataAvailable = -32,
    #[error("Invalid data")]
    InvalidData = -33,
    #[error("Timeout")]
    Timeout = -34,
    #[error("No network")]
    NoNetwork = -35,
    #[error("Not unique")]
    NotUnique = -36,
    #[error("Not a socket")]
    NotSocket = -37,
    #[error("No address")]
    NoAddress = -38,
    #[error("Bad protocol")]
    BadProtocol = -39,
    #[error("Protocol unavailable")]
    ProtocolUnavailable = -40,
    #[error("Protocol not supported")]
    ProtocolNotSupported = -41,
    #[error("Protocol family not supported")]
    ProtocolFamilyNotSupported = -42,
    #[error("Address family not supported")]
    AddressFamilyNotSupported = -43,
    #[error("Socket not supported")]
    SocketNotSupported = -44,
    #[error("Connection reset")]
    ConnectionReset = -45,
    #[error("Already connected")]
    AlreadyConnected = -46,
    #[error("Not connected")]
    NotConnected = -47,
    #[error("Connection refused")]
    ConnectionRefused = -48,
    #[error("No host")]
    NoHost = -49,
    #[error("In progress")]
    InProgress = -50,
    #[error("Cancelled")]
    Cancelled = -51,
    #[error("Memory already mapped")]
    MemoryAlreadyMapped = -52,
    #[error("CRC mismatch")]
    CrcMismatch = -100,
    #[error("Format not supported")]
    FormatNotSupported = -200,
    #[error("Device type not supported")]
    DeviceTypeNotSupported = -201,
    #[error("Share mode not supported")]
    ShareModeNotSupported = -202,
    #[error("No backend")]
    NoBackend = -203,
    #[error("No device")]
    NoDevice = -204,
    #[error("API not found")]
    ApiNotFound = -205,
    #[error("Invalid device config")]
    InvalidDeviceConfig = -206,
    #[error("Loop")]
    Loop = -207,
    #[error("Backend not enabled")]
    BackendNotEnabled = -208,
    #[error("Device not initialized")]
    DeviceNotInitialized = -300,
    #[error("Device already initialized")]
    DeviceAlreadyInitialized = -301,
    #[error("Device not started")]
    DeviceNotStarted = -302,
    #[error("Device not stopped")]
    DeviceNotStopped = -303,
    #[error("Failed to init backend")]
    FailedToInitBackend = -400,
    #[error("Failed to open backend device")]
    FailedToOpenBackendDevice = -401,
    #[error("Failed to start backend device")]
    FailedToStartBackendDevice = -402,
    #[error("Failed to stop backend device")]
    FailedToStopBackendDevice = -403,
}

impl Error {
    pub fn from_i32(code: i32) -> Result<(), Error> {
        match code {
            0 => Ok(()),
            -1 => Err(Error::Error),
            -2 => Err(Error::InvalidArgs),
            -3 => Err(Error::InvalidOperation),
            -4 => Err(Error::OutOfMemory),
            -5 => Err(Error::OutOfRange),
            -6 => Err(Error::AccessDenied),
            -7 => Err(Error::DoesNotExist),
            -8 => Err(Error::AlreadyExists),
            -9 => Err(Error::TooManyOpenFiles),
            -10 => Err(Error::InvalidFile),
            -11 => Err(Error::TooBig),
            -12 => Err(Error::PathTooLong),
            -13 => Err(Error::NameTooLong),
            -14 => Err(Error::NotDirectory),
            -15 => Err(Error::IsDirectory),
            -16 => Err(Error::DirectoryNotEmpty),
            -17 => Err(Error::AtEnd),
            -18 => Err(Error::NoSpace),
            -19 => Err(Error::Busy),
            -20 => Err(Error::IOError),
            -21 => Err(Error::Interrupt),
            -22 => Err(Error::Unavailable),
            -23 => Err(Error::AlreadyInUse),
            -24 => Err(Error::BadAddress),
            -25 => Err(Error::BadSeek),
            -26 => Err(Error::BadPipe),
            -27 => Err(Error::Deadlock),
            -28 => Err(Error::TooManyLinks),
            -29 => Err(Error::NotImplemented),
            -30 => Err(Error::NoMessage),
            -31 => Err(Error::BadMessage),
            -32 => Err(Error::NoDataAvailable),
            -33 => Err(Error::InvalidData),
            -34 => Err(Error::Timeout),
            -35 => Err(Error::NoNetwork),
            -36 => Err(Error::NotUnique),
            -37 => Err(Error::NotSocket),
            -38 => Err(Error::NoAddress),
            -39 => Err(Error::BadProtocol),
            -40 => Err(Error::ProtocolUnavailable),
            -41 => Err(Error::ProtocolNotSupported),
            -42 => Err(Error::ProtocolFamilyNotSupported),
            -43 => Err(Error::AddressFamilyNotSupported),
            -44 => Err(Error::SocketNotSupported),
            -45 => Err(Error::ConnectionReset),
            -46 => Err(Error::AlreadyConnected),
            -47 => Err(Error::NotConnected),
            -48 => Err(Error::ConnectionRefused),
            -49 => Err(Error::NoHost),
            -50 => Err(Error::InProgress),
            -51 => Err(Error::Cancelled),
            -52 => Err(Error::MemoryAlreadyMapped),
            -100 => Err(Error::CrcMismatch),
            -200 => Err(Error::FormatNotSupported),
            -201 => Err(Error::DeviceTypeNotSupported),
            -202 => Err(Error::ShareModeNotSupported),
            -203 => Err(Error::NoBackend),
            -204 => Err(Error::NoDevice),
            -205 => Err(Error::ApiNotFound),
            -206 => Err(Error::InvalidDeviceConfig),
            -207 => Err(Error::Loop),
            -208 => Err(Error::BackendNotEnabled),
            -300 => Err(Error::DeviceNotInitialized),
            -301 => Err(Error::DeviceAlreadyInitialized),
            -302 => Err(Error::DeviceNotStarted),
            -303 => Err(Error::DeviceNotStopped),
            -400 => Err(Error::FailedToInitBackend),
            -401 => Err(Error::FailedToOpenBackendDevice),
            -402 => Err(Error::FailedToStartBackendDevice),
            -403 => Err(Error::FailedToStopBackendDevice),
            _ => Err(Error::Error), // Fallback to generic error
        }
    }

    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}
