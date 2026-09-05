use crate::protocol::ProtocolVersion;

/// Manufacturer of the Bluetooth cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Manufacturer {
    Gan,
    MoYu,
    MonsterGo,
    Xiaomi,
    Particula,
    QiYi,
}

/// Feature capabilities of a Bluetooth cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CubeFeature {
    /// Hardware gyroscope for orientation tracking
    Gyroscope,
    /// Battery level reporting
    Battery,
    /// Hardware info query (firmware version, etc.)
    HardwareInfo,
    /// Move history recovery (lost BLE packet detection)
    MoveHistory,
}

/// Verification status of a cube definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CubeStatus {
    /// Tested and confirmed working with physical hardware
    Verified,
    /// Not tested — awaiting community confirmation
    Unverified,
}

/// Definition of a known Bluetooth smart cube model.
#[derive(Debug, Clone)]
pub struct CubeDefinition {
    /// Commercial name (e.g. "GAN 12 ui FreePlay")
    pub name: &'static str,
    /// Manufacturer
    pub manufacturer: Manufacturer,
    /// BLE advertised name prefixes used to identify this cube during scanning
    pub ble_name_prefixes: &'static [&'static str],
    /// Protocol version used by this cube
    pub protocol: ProtocolVersion,
    /// Supported features
    pub features: &'static [CubeFeature],
    /// Verification status (tested with physical hardware?)
    pub status: CubeStatus,
}

// =============================================================================
// GAN v1 cubes (original 356i series)
// =============================================================================

pub const GAN356I: CubeDefinition = CubeDefinition {
    name: "GAN 356i",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV1,
    features: &[CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Unverified,
};

pub const GAN356I_PLAY: CubeDefinition = CubeDefinition {
    name: "GAN 356i Play",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV1,
    features: &[CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Unverified,
};

pub const GAN356I_2: CubeDefinition = CubeDefinition {
    name: "GAN 356i 2",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV1,
    features: &[CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Unverified,
};

pub const GAN356I_2_PLAY: CubeDefinition = CubeDefinition {
    name: "GAN 356i 2 Play",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV1,
    features: &[CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Unverified,
};

// =============================================================================
// GAN Gen2 cubes
// =============================================================================

pub const GAN_MINI_UI_FREEPLAY: CubeDefinition = CubeDefinition {
    name: "GAN Mini ui FreePlay",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV2,
    features: &[CubeFeature::Gyroscope, CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Unverified,
};

pub const GAN12_UI_FREEPLAY: CubeDefinition = CubeDefinition {
    name: "GAN 12 ui FreePlay",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV2,
    features: &[CubeFeature::Gyroscope, CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Unverified,
};

pub const GAN12_UI: CubeDefinition = CubeDefinition {
    name: "GAN 12 ui",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV2,
    features: &[CubeFeature::Gyroscope, CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Unverified,
};

pub const GAN356_I_CARRY_S: CubeDefinition = CubeDefinition {
    name: "GAN 356 i Carry S",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV2,
    features: &[CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Unverified,
};

pub const GAN356_I_CARRY: CubeDefinition = CubeDefinition {
    name: "GAN 356 i Carry",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV2,
    features: &[CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Unverified,
};

pub const GAN356_I_CARRY_E: CubeDefinition = CubeDefinition {
    name: "GAN 356 i Carry E",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV2,
    features: &[CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Unverified,
};

pub const GAN356_I3: CubeDefinition = CubeDefinition {
    name: "GAN 356 i 3",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV2,
    features: &[CubeFeature::Gyroscope, CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Unverified,
};

pub const MONSTER_GO_3AI: CubeDefinition = CubeDefinition {
    name: "Monster Go 3Ai",
    manufacturer: Manufacturer::MonsterGo,
    ble_name_prefixes: &["MG"],
    protocol: ProtocolVersion::GanV2,
    features: &[CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Unverified,
};

// =============================================================================
// GAN Gen3 cubes
// =============================================================================

pub const GAN356_I_CARRY_2: CubeDefinition = CubeDefinition {
    name: "GAN 356 i Carry 2",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV3,
    features: &[CubeFeature::Battery, CubeFeature::HardwareInfo, CubeFeature::MoveHistory],
    status: CubeStatus::Unverified,
};

// =============================================================================
// GAN Gen4 cubes
// =============================================================================

pub const GAN12_UI_MAGLEV: CubeDefinition = CubeDefinition {
    name: "GAN 12 ui Maglev",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV4,
    features: &[CubeFeature::Gyroscope, CubeFeature::Battery, CubeFeature::HardwareInfo, CubeFeature::MoveHistory],
    status: CubeStatus::Unverified,
};

pub const GAN14_UI_FREEPLAY: CubeDefinition = CubeDefinition {
    name: "GAN 14 ui FreePlay",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV4,
    features: &[CubeFeature::Battery, CubeFeature::HardwareInfo, CubeFeature::MoveHistory],
    status: CubeStatus::Unverified,
};

pub const GAN356_I_CARRY_4: CubeDefinition = CubeDefinition {
    name: "GAN 356 i Carry 4",
    manufacturer: Manufacturer::Gan,
    ble_name_prefixes: &["GAN"],
    protocol: ProtocolVersion::GanV4,
    features: &[CubeFeature::Battery, CubeFeature::HardwareInfo, CubeFeature::MoveHistory],
    status: CubeStatus::Unverified,
};

// =============================================================================
// MoYu cubes
// =============================================================================

pub const MOYU_AI_2023: CubeDefinition = CubeDefinition {
    name: "MoYu AI 2023",
    manufacturer: Manufacturer::MoYu,
    ble_name_prefixes: &["AiCube"],
    protocol: ProtocolVersion::MoYuAi,
    features: &[CubeFeature::Gyroscope, CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Unverified,
};

pub const MOYU_AI_V2: CubeDefinition = CubeDefinition {
    name: "MoYu AI v2",
    manufacturer: Manufacturer::MoYu,
    ble_name_prefixes: &["MHC"],
    protocol: ProtocolVersion::MoYuAi,
    features: &[CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Unverified,
};

pub const MOYU_WEILONG_V10: CubeDefinition = CubeDefinition {
    name: "MoYu WeiLong V10",
    manufacturer: Manufacturer::MoYu,
    ble_name_prefixes: &["WCU_MY"],
    protocol: ProtocolVersion::MoYuV3,
    features: &[CubeFeature::Gyroscope, CubeFeature::Battery, CubeFeature::HardwareInfo],
    status: CubeStatus::Verified,
};

// =============================================================================
// Xiaomi Giiker cubes
// =============================================================================

pub const GIIKER_I3: CubeDefinition = CubeDefinition {
    name: "Giiker i3",
    manufacturer: Manufacturer::Xiaomi,
    ble_name_prefixes: &["GiC"],
    protocol: ProtocolVersion::GiikerV1,
    features: &[CubeFeature::Battery],
    status: CubeStatus::Unverified,
};

pub const GIIKER_I3S: CubeDefinition = CubeDefinition {
    name: "Giiker i3S",
    manufacturer: Manufacturer::Xiaomi,
    ble_name_prefixes: &["GiS"],
    protocol: ProtocolVersion::GiikerV1,
    features: &[CubeFeature::Battery],
    status: CubeStatus::Unverified,
};

pub const GIIKER_I3Y: CubeDefinition = CubeDefinition {
    name: "Giiker i3Y",
    manufacturer: Manufacturer::Xiaomi,
    ble_name_prefixes: &["Gi"],
    protocol: ProtocolVersion::GiikerV1,
    features: &[CubeFeature::Battery],
    status: CubeStatus::Unverified,
};

pub const MI_SMART_MAGIC_CUBE: CubeDefinition = CubeDefinition {
    name: "Mi Smart Magic Cube",
    manufacturer: Manufacturer::Xiaomi,
    ble_name_prefixes: &["Mi Smart Magic Cube"],
    protocol: ProtocolVersion::GiikerV1,
    features: &[CubeFeature::Battery],
    status: CubeStatus::Unverified,
};

// =============================================================================
// Particula / Rubik's cubes
// =============================================================================

pub const GOCUBE: CubeDefinition = CubeDefinition {
    name: "GoCube",
    manufacturer: Manufacturer::Particula,
    ble_name_prefixes: &["GoCube"],
    protocol: ProtocolVersion::GoCube,
    features: &[CubeFeature::Battery],
    status: CubeStatus::Unverified,
};

pub const RUBIKS_CONNECTED: CubeDefinition = CubeDefinition {
    name: "Rubik's Connected",
    manufacturer: Manufacturer::Particula,
    ble_name_prefixes: &["Rubiks"],
    protocol: ProtocolVersion::GoCube,
    features: &[CubeFeature::Battery],
    status: CubeStatus::Unverified,
};

// =============================================================================
// QiYi cubes
// =============================================================================

pub const QIYI_TORNADO_V4_SCS: CubeDefinition = CubeDefinition {
    name: "QiYi Tornado V4 (SCS)",
    manufacturer: Manufacturer::QiYi,
    ble_name_prefixes: &["QY-QYSC"],
    protocol: ProtocolVersion::QiYiSmart,
    features: &[CubeFeature::Battery],
    status: CubeStatus::Unverified,
};

pub const QIYI_TORNADO_V4_AI: CubeDefinition = CubeDefinition {
    name: "QiYi Tornado V4 AI",
    manufacturer: Manufacturer::QiYi,
    ble_name_prefixes: &["XMD-TornadoV4-i-"],
    protocol: ProtocolVersion::QiYiSmart,
    features: &[CubeFeature::Battery],
    status: CubeStatus::Unverified,
};

pub const QIYI_AI_3X3: CubeDefinition = CubeDefinition {
    name: "QiYi AI 3x3",
    manufacturer: Manufacturer::QiYi,
    ble_name_prefixes: &["QY-"],
    protocol: ProtocolVersion::QiYiSmart,
    features: &[CubeFeature::Battery],
    status: CubeStatus::Unverified,
};

// =============================================================================
// Catalogue
// =============================================================================

/// All known Bluetooth smart cubes.
pub const KNOWN_CUBES: &[CubeDefinition] = &[
    // GAN v1
    GAN356I,
    GAN356I_PLAY,
    GAN356I_2,
    GAN356I_2_PLAY,
    // GAN Gen2
    GAN_MINI_UI_FREEPLAY,
    GAN12_UI_FREEPLAY,
    GAN12_UI,
    GAN356_I_CARRY_S,
    GAN356_I_CARRY,
    GAN356_I_CARRY_E,
    GAN356_I3,
    MONSTER_GO_3AI,
    // GAN Gen3
    GAN356_I_CARRY_2,
    // GAN Gen4
    GAN12_UI_MAGLEV,
    GAN14_UI_FREEPLAY,
    GAN356_I_CARRY_4,
    // MoYu
    MOYU_AI_2023,
    MOYU_AI_V2,
    MOYU_WEILONG_V10,
    // Xiaomi Giiker
    GIIKER_I3,
    GIIKER_I3S,
    GIIKER_I3Y,
    MI_SMART_MAGIC_CUBE,
    // Particula / Rubik's
    GOCUBE,
    RUBIKS_CONNECTED,
    // QiYi
    QIYI_TORNADO_V4_SCS,
    QIYI_TORNADO_V4_AI,
    QIYI_AI_3X3,
];

/// Find a cube definition by BLE advertised device name.
///
/// Uses longest-prefix match for specificity. E.g. "AiCube" matches
/// MoYu AI 2023 before falling back to generic "GAN" prefixes, and
/// "Mi Smart Magic Cube" matches before "Mi".
pub fn find_cube_by_ble_name(device_name: &str) -> Option<&'static CubeDefinition> {
    let mut best_match: Option<(&CubeDefinition, usize)> = None;

    for cube in KNOWN_CUBES {
        for prefix in cube.ble_name_prefixes {
            if device_name.starts_with(prefix) {
                let len = prefix.len();
                if best_match.map_or(true, |(_, best_len)| len > best_len) {
                    best_match = Some((cube, len));
                }
            }
        }
    }

    best_match.map(|(cube, _)| cube)
}

/// Find all cube definitions that use a given protocol version.
pub fn find_cubes_by_protocol(protocol: ProtocolVersion) -> Vec<&'static CubeDefinition> {
    KNOWN_CUBES.iter().filter(|c| c.protocol == protocol).collect()
}

/// Get all unique BLE service UUIDs to scan for.
pub fn all_scan_service_uuids() -> Vec<&'static str> {
    let mut uuids: Vec<&str> = Vec::new();
    for cube in KNOWN_CUBES {
        let uuid = cube.protocol.ble_profile().service_uuid;
        if !uuids.contains(&uuid) {
            uuids.push(uuid);
        }
    }
    uuids
}

/// Get all unique BLE name prefixes to filter for during scanning.
pub fn all_scan_name_prefixes() -> Vec<&'static str> {
    let mut prefixes: Vec<&str> = Vec::new();
    for cube in KNOWN_CUBES {
        for prefix in cube.ble_name_prefixes {
            if !prefixes.contains(prefix) {
                prefixes.push(prefix);
            }
        }
    }
    prefixes
}
