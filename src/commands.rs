// Size of sesponse without data reference
pub const RESPONSE_SIZE: usize = 10;

/// Error Codes
///
/// See https://github.com/espressif/esptool/blob/67a91cbfef54f281212951b8226583ba3c1d0a85/flasher_stub/include/stub_flasher.h#L95
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Error {
    BadDataLen = 0xC0,
    BadDataChecksum = 0xC1,
    BadBlocksize = 0xC2,
    InvalidCommand = 0xC3,
    FailedSpiOp = 0xC4,
    FailedSpiUnlock = 0xC5,
    NotInFlashMode = 0xC6,
    Inflate = 0xC7,
    NotEnoughData = 0xC8,
    TooMuchData = 0xC9,
    CmdNotImplemented = 0xFF,

    /// Error on esp_rom_spiflash_read
    Err0x63 = 0x63,
    /// Error if the modulus of the flash address to be erased divided by
    /// FLASH_SECTOR_SIZE is not equal to zero.
    Err0x32 = 0x32, // TODO: Never used
    /// Error if the modulus of the flash length to be erased divided by
    /// FLASH_SECTOR_SIZE is not equal to zero.
    Err0x33 = 0x33, // TODO: Never used
    /// Error on esp_rom_spiflash_unlock
    Err0x34 = 0x34, // TODO: Never used
    /// Error in esp_rom_spiflash_erase_sector
    Err0x35 = 0x35, // TODO: Never used

    EraseErr = 0x36, // TODO: Is it OK to add custom Error?
}

/// Command identifier
///
/// https://docs.espressif.com/projects/esptool/en/latest/esp32c3/advanced-topics/serial-protocol.html#supported-by-stub-loader-and-rom-loader
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandCode {
    // Commands supported by the ESP8266 and all the ESP32 variants bootloaders
    FlashBegin = 0x02,
    FlashData = 0x03,
    FlashEnd = 0x04,
    MemBegin = 0x05,
    MemEnd = 0x06,
    MemData = 0x07,
    Sync = 0x08,
    WriteReg = 0x09,
    ReadReg = 0x0A,
    // Commands supported by all the ESP32 variants bootloaders
    SpiSetParams = 0x0B,
    SpiAttach = 0x0D,
    ChangeBaudrate = 0x0F,
    FlashDeflBegin = 0x10,
    FlashDeflData = 0x11,
    FlashDeflEnd = 0x12,
    SpiFlashMd5 = 0x13,
    // Not supported on ESP32
    GetSecurityInfo = 0x14,
    // Stub-only commands
    EraseFlash = 0xD0,
    EraseRegion = 0xD1,
    ReadFlash = 0xD2,
    RunUserCode = 0xD3,
    // Flash encryption debug mode supported command
    FlashEncryptedData = 0xD4,
}

/// Command packet header
///
/// See https://docs.espressif.com/projects/esptool/en/latest/esp32c2/advanced-topics/serial-protocol.html#id2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct CommandBase {
    /// Direction of the packet. Always 0x00 for commands.
    pub direction: Direction,
    /// Command identifier
    pub code: CommandCode,
    /// Length of Data field, in bytes.
    pub size: u16,
    /// Simple checksum of part of the data field (only used for _DATA commands)
    pub checksum: u32,
}

/// SYNC command input data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct SyncCommand {
    pub base: CommandBase,
    /// 36 bytes: 0x07 0x07 0x12 0x20, followed by 32 x 0x55
    pub payload: [u8; 36],
}

/// _BEGIN commands input data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct BeginCommand {
    pub base: CommandBase,
    /// Size to erase
    pub total_size: u32,
    /// Number of data packets
    pub packet_count: u32,
    /// Data size in one packet
    pub packet_size: u32,
    /// Flash offset
    pub offset: u32,
    // In ROM, there also a field to begin encrypted flash.
}

/// _DATA commands input data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct DataCommand {
    pub base: CommandBase,
    /// Data size
    pub size: u32,
    /// Sequence number
    pub sequence_num: u32,
    /// 0, 0
    pub reserved: [u32; 2],
}

/// _END commands input data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct EndFlashCommand {
    pub base: CommandBase,
    /// 0 means reboot, 1 to run user code.
    reboot: bool, // private to ensure should_reboot is used.
}

impl EndFlashCommand {
    pub fn should_reboot(&self) -> bool {
        // As per the logic here: https://github.com/espressif/esptool/blob/0a9caaf04cfde6fd97c785d4811f3fde09b1b71f/flasher_stub/stub_flasher.c#L402
        // 0 means reboot, 1 means do nothing
        !self.reboot
    }
}

/// MEM_END command input data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct MemEndCommand {
    pub base: CommandBase,
    /// Execute flag
    pub stay_in_stub: u32,
    /// Entry point address
    pub entrypoint: fn(),
}

/// WRITE_REG command input data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct WriteRegCommand {
    pub base: CommandBase,
    /// Address
    pub address: u32,
    /// Value
    pub value: u32,
    /// Mask
    pub mask: u32,
    /// Delay in microseconds
    pub delay_us: u32,
}

/// READ_REG command input data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct ReadRegCommand {
    pub base: CommandBase,
    /// Address
    pub address: u32,
}

/// Parameters of the attached SPI flash chip (sizes, etc).
///
/// See https://github.com/espressif/esptool/blob/16e4faeeaa3f95c6b24dfdcc498ffc33924d5f5f/esptool/loader.py#L1214
// TODO: Possibly move to other module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct SpiParams {
    /// Flash chip ID
    pub id: u32,
    /// Total size in bytes
    pub total_size: u32,
    /// Block size
    pub block_size: u32,
    /// Sector size
    pub sector_size: u32,
    /// Page size
    pub page_size: u32,
    /// Status mask
    pub status_mask: u32,
}

/// SPI_SET_PARAMS command input data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct SpiSetParamsCommand {
    pub base: CommandBase,
    /// SPI parameters
    pub params: SpiParams,
}

/// Parameters for attaching to a target devices SPI flash
// TODO: Possibly move to other module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct SpiAttachParams {
    /// CLK pin
    clk: u8,
    /// Q pin
    q: u8,
    /// D pin
    d: u8,
    /// HD pin
    hd: u8,
    /// CS pin
    cs: u8,
}

/// SPI_ATTACH command input data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct SpiAttachCommand {
    pub base: CommandBase,
    /// SPI attach parameters
    pub params: SpiAttachParams,
    // TODO: On ROM loader, there  there is an additional 4 bytes in the data payload of this
    // command. These bytes should all be set to zero.
}

/// CHANGE_BAUDRATE command input data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct ChangeBaudrateCommand {
    pub base: CommandBase,
    /// New baud rate
    pub new: u32,
    /// 0 if we are talking to the ROM loader or the current/old baud rate if we
    /// are talking to the stub loader.
    pub old: u32,
}

/// SPI_FLASH_MD5 command input data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct SpiFlashMd5Command {
    pub base: CommandBase,
    // Address
    pub address: u32,
    // Size
    pub size: u32,
    // 0, 0
    pub reserved: [u32; 2],
}

/// ERASE_REGION command input data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct EraseRegionCommand {
    pub base: CommandBase,
    /// Address
    pub address: u32,
    /// Size
    pub size: u32,
}

/// Parameters of ReadFlash
// TODO: Possibly move to other module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct ReadFlashParams {
    /// Flash offset
    pub address: u32,
    /// Read size
    pub total_size: u32,
    /// Size of each individual packet of data
    pub packet_size: u32,
    /// Maximum number of un-acked packets
    pub max_in_flight: u32,
}

/// READ_FLASH command input data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct ReadFlashCommand {
    pub base: CommandBase,
    /// Read Flash parameters
    pub params: ReadFlashParams,
}

/// Direction of the packet
#[allow(unused)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Requests
    In = 0x00,
    /// Responses
    Out = 0x01,
}

/// Response packet
///
/// See https://docs.espressif.com/projects/esptool/en/latest/esp32c2/advanced-topics/serial-protocol.html#id3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed(1))]
pub struct Response<'a> {
    /// Direction of the packet. Always 0x01 for responses.
    pub direction: Direction,
    /// Same value as Command identifier in the request packet that triggered
    /// the response
    pub command: CommandCode,
    /// Size of data field. At least the length of the Status Bytes
    pub size: u16,
    /// Response value used by READ_REG command. Zero otherwise.
    pub value: u32,
    /// Status flag, success (0) or failure (1)
    pub status: u8,
    /// If Status is 1, this indicates the type of error.
    pub error: u8,
    // TODO: ROM loader has  4 bytes status (https://docs.espressif.com/projects/esptool/en/latest/esp32c2/advanced-topics/serial-protocol.html#status-bytes)
    /// Variable length data payload. Length indicated by `size` field.
    pub data: &'a [u8],
}

impl<'a> Response<'a> {
    pub fn new(cmd: CommandCode) -> Self {
        Response {
            direction: Direction::Out,
            command: cmd,
            size: 2,
            value: 0,
            status: 0,
            error: 0,
            data: &[],
        }
    }

    pub fn value(&mut self, value: u32) {
        self.value = value;
    }

    pub fn data(&mut self, data: &'a [u8]) {
        self.size = 2 + data.len() as u16;
        self.data = data;
    }

    pub fn error(&mut self, error: Error) {
        crate::dprintln!("Error: {:?}", error);
        self.status = 1;
        self.error = error as u8;
    }
}
