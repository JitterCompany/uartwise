
// mod interface;
use crate::interface::DisplayInterface;
// use interface::DisplayInterface;

/// SSD1362 Commands
/// Based on the command table from the OEL9M1020-O-E datasheet
///


/// Commands
#[derive(Debug)]
#[allow(dead_code)]
pub enum Command {
    /// Setup column start and end address
    /// values range from 0-127 (0x00h - 0x7F)
    /// This is only for horizontal or vertical addressing mode
    ColumnAddress(u8, u8),

    /// Setup row start and end address
    /// values range from 0-63 (0x00h - 0x3F)
    /// This is only for horizontal or vertical addressing mode
    RowAddress(u8, u8),

    /// Set Contrast Control.
    /// Higher number is higher contrast. Default = 0x7F
    Contrast(u8),

    /// Set Re-map
    /// Column Address Re-map, Nibble Re-map, Horizontal Address, COM Re-map, SEG Split Odd Even, SEG left/right remap
    // Remap(bool, bool, bool, bool, bool, bool),
    Remap(u8), // just a single value for now

    /// Set display start line
    /// Vertical shift by setting the starting address of display RAM from 0 ~ 63
    StartLine(u8),

    /// Set vertical offset by COM from 0 ~ 63 (RESET = 00h)
    DisplayOffset(u8),

    /// Setup vertical scroll area.
    /// Values are number of rows above scroll area (0-63)
    /// and number of rows of scrolling. (0-64)
    VScrollArea(u8, u8),

    /// MODE
    Mode(DisplayMode),

    /// Set multipex ratio from 3-63 (MUX-1)
    Multiplex(u8),

    /// Vdd Select. Select true for internal VDD, false for external
    InternalVDD(bool),

    /// I_Ref Select. Select true for internal I_REF, false for external
    InternalIREF(bool),

    /// Turn display on or off.
    DisplayOn(bool),

    /// PWM Phase length selection Phaswe 1 and Phase 2
    PhaseLength(u8),

    /// Set up display clock.
    /// First value is oscillator frequency, increasing with higher value
    /// Second value is divide ratio - 1
    DisplayClockDiv(u8, u8),

     /// Set second precharge period. each value is from 1-15
    PreChargePeriod(u8),

    /// GrayScale - configure 16 levels
    // GrayScale()

    /// linear LUT
    DefaultGrayScale(),

    /// Set Pre-charge voltage level 0 - 0x1F
    /// 0.10 * Vcc - 0.51 * vcc
    PreChargeVoltage(u8),

    /// Pre-charge voltage capacitor selection
    /// false = without external Vp capacitor
    /// true = with external Vp capacitor
    PreChargeCapacitor(bool),

    /// Set Vcomh Deselect level
    VcomhDeselect(VcomhLevel),

    /// MCU protection status.
    /// If True: Lock OLED driver IC MCU interface from entering command
    CommandLock(bool),

    // FadeLevel(u8),

    // Blink(u8),
}

impl Command {
    /// Send command to SSD1362
    pub fn send<DI>(self, iface: &mut DI) -> Result<(), DI::Error>
    where
        DI: DisplayInterface,
    {

        // Transform command into a fixed size array of 7 u8 and the real length for sending
        let (data, len) = match self {
            Command::ColumnAddress(start, end) => ([0x15, start, end, 0, 0, 0, 0], 3),
            Command::RowAddress(start, end) => ([0x75, start, end, 0, 0, 0, 0], 3),
            Command::Contrast(val) => ([0x81, val, 0, 0, 0, 0, 0], 2),
            Command::Remap(remap) => ([0xA0, remap, 0, 0, 0, 0, 0], 2),
            Command::StartLine(line) => ([0xA1, line, 0, 0, 0, 0, 0], 2),
            Command::DisplayOffset(offset) => ([0xA2, offset, 0, 0, 0, 0, 0], 2),
            Command::VScrollArea(above, lines) => ([0xA3, above, lines, 0, 0, 0, 0], 3),
            Command::Mode(mode) => ([mode as u8, 0, 0, 0, 0, 0, 0], 1),
            Command::Multiplex(ratio) => ([0xA8, ratio, 0, 0, 0, 0, 0], 2),
            Command::InternalVDD(en) => ([0xAB, en as u8, 0, 0, 0, 0, 0], 2),
            Command::InternalIREF(en) => ([0xAD, (en as u8) << 4 | 0x8E, 0, 0, 0, 0, 0], 2),
            Command::DisplayOn(on) => ([0xAE | (on as u8), 0, 0, 0, 0, 0, 0], 1),
            Command::PhaseLength(len) => ([0xB1, len, 0, 0, 0, 0, 0], 2),
            Command::DisplayClockDiv(fosc, div) => {
                ([0xB3, ((0xF & fosc) << 4) | (0xF & div), 0, 0, 0, 0, 0], 2)
            },
            Command::PreChargePeriod(period) => ([0xB6, period, 0, 0, 0, 0, 0], 2),
            Command::DefaultGrayScale()=> ([0xB9, 0, 0, 0, 0, 0, 0], 2),
            Command::PreChargeVoltage(vol) => ([0xBC, vol, 0, 0, 0, 0, 0], 2),
            Command::PreChargeCapacitor(cap) => ([0xBD, cap as u8, 0, 0, 0, 0, 0], 2),
            Command::VcomhDeselect(level) => ([0xBE, (level as u8), 0, 0, 0, 0, 0], 2),
            Command::CommandLock(lock) => ([0xFD, ((lock as u8) & 0x1 << 2) | 0x12, 0, 0, 0, 0, 0], 2),
        };

        // Send command over the interface
        iface.send_commands(&data[0..len])
    }
}



/// Vcomh Deselect level
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum DisplayMode {
    Normal = 0x4,
    AllOn = 0x5,
    AllOff = 0x6,
    Inverse = 0x7
}

/// Vcomh Deselect level
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum VcomhLevel {
    /// 0.72 * Vcc
    V072 = 0b000,
    /// 0.77 * Vcc
    V082 = 0b101,
    /// 0.83 * Vcc
    V086 = 0b111
}