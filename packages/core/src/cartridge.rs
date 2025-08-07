use crate::{cpu::BusModule, utils::array};

static CARTRIDGE_TYPE: [&'static str; 256] = array!["Unknown"; 256;
  [0x00] = "ROM ONLY",
  [0x01] = "MBC1",
  [0x02] = "MBC1+RAM",
  [0x03] = "MBC1+RAM+BATTERY",
  [0x05] = "MBC2",
  [0x06] = "MBC2+BATTERY",
  [0x08] = "ROM+RAM 9",
  [0x09] = "ROM+RAM+BATTERY 9",
  [0x0B] = "MMM01",
  [0x0C] = "MMM01+RAM",
  [0x0D] = "MMM01+RAM+BATTERY",
  [0x0F] = "MBC3+TIMER+BATTERY",
  [0x10] = "MBC3+TIMER+RAM+BATTERY 10",
  [0x11] = "MBC3",
  [0x12] = "MBC3+RAM 10",
  [0x13] = "MBC3+RAM+BATTERY 10",
  [0x19] = "MBC5",
  [0x1A] = "MBC5+RAM",
  [0x1B] = "MBC5+RAM+BATTERY",
  [0x1C] = "MBC5+RUMBLE",
  [0x1D] = "MBC5+RUMBLE+RAM",
  [0x1E] = "MBC5+RUMBLE+RAM+BATTERY",
  [0x20] = "MBC6",
  [0x22] = "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
  [0xFC] = "POCKET CAMERA",
  [0xFD] = "BANDAI TAMA5",
  [0xFE] = "HuC3",
  [0xFF] = "HuC1+RAM+BATTERY",
];

fn cartridge_type_name(header: &RomHeader) -> &'static str {
    match header.cart_type {
        cart_type if (cart_type as usize) < CARTRIDGE_TYPE.len() => {
            CARTRIDGE_TYPE[cart_type as usize]
        }
        _ => "Unknown",
    }
}

macro_rules! new_licensee_code_array {
    ($def:expr; $len:expr; $([$idx1:expr,$idx2:expr]=$val:expr),* $(,)?) => { {
        let mut a = [$def; $len];
        $(
            a[($idx2 as usize) << 8 | ($idx1 as usize)] = $val;
        )*
        a
    } }
}

static NEW_LICENSEE_CODE: [&'static str; 65535] = new_licensee_code_array!["Unknown"; 65535;
  ['0', '0'] =	"None",
  ['0', '1'] =	"Nintendo Research & Development 1",
  ['0', '8'] =	"Capcom",
  ['1', '3'] =	"EA (Electronic Arts)",
  ['1', '8'] =	"Hudson Soft",
  ['1', '9'] =	"B-AI",
  ['2', '0'] =	"KSS",
  ['2', '2'] =	"Planning Office WADA",
  ['2', '4'] =	"PCM Complete",
  ['2', '5'] =	"San-X",
  ['2', '8'] =	"Kemco",
  ['2', '9'] =	"SETA Corporation",
  ['3', '0'] =	"Viacom",
  ['3', '1'] =	"Nintendo",
  ['3', '2'] =	"Bandai",
  ['3', '3'] =	"Ocean Software/Acclaim Entertainment",
  ['3', '4'] =	"Konami",
  ['3', '5'] =	"HectorSoft",
  ['3', '7'] =	"Taito",
  ['3', '8'] =	"Hudson Soft",
  ['3', '9'] =	"Banpresto",
  ['4', '1'] =	"Ubi Soft1",
  ['4', '2'] =	"Atlus",
  ['4', '4'] =	"Malibu Interactive",
  ['4', '6'] =	"Angel",
  ['4', '7'] =	"Bullet-Proof Software2",
  ['4', '9'] =	"Irem",
  ['5', '0'] =	"Absolute",
  ['5', '1'] =	"Acclaim Entertainment",
  ['5', '2'] =	"Activision",
  ['5', '3'] =	"Sammy USA Corporation",
  ['5', '4'] =	"Konami",
  ['5', '5'] =	"Hi Tech Expressions",
  ['5', '6'] =	"LJN",
  ['5', '7'] =	"Matchbox",
  ['5', '8'] =	"Mattel",
  ['5', '9'] =	"Milton Bradley Company",
  ['6', '0'] =	"Titus Interactive",
  ['6', '1'] =	"Virgin Games Ltd.3",
  ['6', '4'] =	"Lucasfilm Games4",
  ['6', '7'] =	"Ocean Software",
  ['6', '9'] =	"EA (Electronic Arts)",
  ['7', '0'] =	"Infogrames5",
  ['7', '1'] =	"Interplay Entertainment",
  ['7', '2'] =	"Broderbund",
  ['7', '3'] =	"Sculptured Software6",
  ['7', '5'] =	"The Sales Curve Limited7",
  ['7', '8'] =	"THQ",
  ['7', '9'] =	"Accolade",
  ['8', '0'] =	"Misawa Entertainment",
  ['8', '3'] =	"lozc",
  ['8', '6'] =	"Tokuma Shoten",
  ['8', '7'] =	"Tsukuda Original",
  ['9', '1'] =	"Chunsoft Co.8",
  ['9', '2'] =	"Video System",
  ['9', '3'] =	"Ocean Software/Acclaim Entertainment",
  ['9', '5'] =	"Varie",
  ['9', '6'] =	"Yonezawa/s’pal",
  ['9', '7'] =	"Kaneko",
  ['9', '9'] =	"Pack-In-Video",
  ['A', '4'] =	"Konami (Yu-Gi-Oh!)",
];

fn cartridge_licensee_name(header: &RomHeader) -> &'static str {
    match header.new_licensee_code {
        [code1, code2] if ((code2 as usize) << 8 | code1 as usize) < NEW_LICENSEE_CODE.len() => {
            NEW_LICENSEE_CODE[(code2 as usize) << 8 | code1 as usize]
        }
        _ => "Unknown",
    }
}

#[repr(C)]
pub struct RomHeader {
    entry: [u8; 4],
    logo: [u8; 0x30],
    title: [u8; 16],
    new_licensee_code: [u8; 2],
    sgb_flag: u8,
    cart_type: u8,
    rom_size: u8,
    ram_size: u8,
    dest_code: u8,
    old_licensee_code: u8,
    version: u8,
    checksum: u8,
    global_checksum: u16,
}

impl RomHeader {
    pub fn title_str(&self) -> String {
        String::from_utf8_lossy(&self.title).to_string()
    }
}

impl std::fmt::Debug for RomHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RomHeader")
            .field("title_str", &self.title_str())
            .field("type", &format!("{:x?}", &self.cart_type))
            .field("new_licensee_code", &self.new_licensee_code)
            .field("licensee_name", &cartridge_licensee_name(&self))
            .field("rom_size", &self.rom_size)
            .field("ram_size", &self.ram_size)
            .field("old_licensee_code", &self.old_licensee_code)
            .field("checksum", &self.checksum)
            .finish()
    }
}

/**
 * 卡带
 */
pub struct Cartridge {
    pub data: Vec<u8>,
}

impl std::fmt::Debug for Cartridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cartridge")
            .field("header", &self.as_header())
            .field("is_checksum_matched", &self.is_checksum_match())
            .finish()
    }
}

impl Cartridge {
    pub fn from(data: Vec<u8>) -> Self {
        Cartridge { data }
    }

    pub fn as_header<'s>(&'s self) -> &'s RomHeader {
        unsafe { &*((self.data.as_ptr().offset(0x100)) as *const RomHeader) }
    }

    pub fn is_checksum_match(&self) -> bool {
        let mut x: u16 = 0;
        for i in 0x0134..=0x014C {
            x = u16::wrapping_sub(x, (self.data[i] as u16) + 1);
        }
        ((x & 0xFF) as u8) == self.as_header().checksum
    }
}

impl BusModule for Cartridge {
    fn read(&self, address: u16) -> u8 {
        // for now just ROM ONLY

        self.data[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {
        // for now ROM ONLY

        // unimplemented!();
    }
}
