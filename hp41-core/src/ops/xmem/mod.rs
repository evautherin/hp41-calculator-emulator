pub mod ops;

use serde::{Deserialize, Serialize};

/// X-MEM file kind: PROGRAM or DATA.
///
/// D-51.0a ISOLATION INVARIANT: X-MEM files have no connection to
/// `state.regs` or `adv_matrices`. X-MEM ops MUST NEVER read/write
/// `state.regs` except via the explicit SAVED/GETD transfer.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum XmemKind {
    /// Program file — data bytes are encoded via `encode_program`.
    #[default]
    Program,
    /// Data file — data bytes are encoded via `encode_data`.
    Data,
}

/// Named extended-memory file for the HP-41CX X-MEM model (P56 / D-51.0a).
///
/// D-51.0a ISOLATION INVARIANT: This struct has no connection to
/// `state.regs` or `adv_matrices`. X-MEM ops MUST NEVER read/write
/// `state.regs` except via the explicit SAVED/GETD transfer.
///
/// Mirrors the `AdvMatrix` precedent (hp41-core/src/ops/advantage/mod.rs:111)
/// in derive attributes and overall shape.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct XmemFile {
    /// ALPHA-register file name identifying this X-MEM file (D-51.0c).
    pub name: String,
    /// File kind: PROGRAM or DATA.
    pub kind: XmemKind,
    /// Raw encoded bytes: PROGRAM = `encode_program` output; DATA = `encode_data` output.
    pub data: Vec<u8>,
    /// DATA-file register count stored at SAVED time (Research Pitfall 1).
    /// Avoids decoding on every EMROOM call. Default 0 for PROGRAM files.
    #[serde(default)]
    pub reg_count: usize,
}

impl XmemFile {
    /// Return the number of X-MEM registers this file occupies (D-51.2).
    ///
    /// - PROGRAM: `⌈data.len() / 7⌉ + 1` (7 program bytes per register, +1 header)
    /// - DATA: `reg_count + 1` (N data registers + 1 header register)
    ///
    /// Uses `div_ceil` (stable on MSRV 1.88). NEVER uses `floor()` or `fmod()`.
    pub fn register_count(&self) -> usize {
        match self.kind {
            XmemKind::Program => self.data.len().div_ceil(7) + 1,
            XmemKind::Data => self.reg_count + 1,
        }
    }
}

/// Total X-MEM register capacity across all installed modules:
/// 124 built-in + 2×238 (two HP 82181A Extended Memory modules).
pub const XMEM_CAPACITY: usize = 600;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Catches: XmemFile default shape regression
    #[test]
    fn xmem_file_default_shape() {
        let f = XmemFile::default();
        assert!(f.name.is_empty(), "default name should be empty");
        assert_eq!(f.kind, XmemKind::Program, "default kind should be Program");
        assert!(f.data.is_empty(), "default data should be empty");
        assert_eq!(f.reg_count, 0, "default reg_count should be 0");
    }

    // Catches: register_count() regression for default (Program, empty data)
    #[test]
    fn register_count_default_program() {
        let f = XmemFile::default();
        // 0.div_ceil(7) == 0, +1 header = 1
        assert_eq!(f.register_count(), 1);
    }

    // Catches: register_count() regression for Program file with 8 bytes
    #[test]
    fn register_count_program_8_bytes() {
        let f = XmemFile {
            data: vec![0u8; 8],
            ..Default::default()
        };
        // 8.div_ceil(7) == 2, +1 header = 3
        assert_eq!(f.register_count(), 3);
    }

    // Catches: register_count() regression for 922-byte Program file (OM-verified example)
    #[test]
    fn register_count_program_922_bytes() {
        let f = XmemFile {
            data: vec![0u8; 922],
            ..Default::default()
        };
        // 922.div_ceil(7) == 132, +1 header = 133
        assert_eq!(f.register_count(), 133);
    }

    // Catches: register_count() regression for Data file with reg_count 5
    #[test]
    fn register_count_data_5_regs() {
        let f = XmemFile {
            kind: XmemKind::Data,
            reg_count: 5,
            ..Default::default()
        };
        // 5 + 1 header = 6
        assert_eq!(f.register_count(), 6);
    }

    // Catches: XMEM_CAPACITY value regression
    #[test]
    fn xmem_capacity_value() {
        assert_eq!(XMEM_CAPACITY, 600, "XMEM_CAPACITY must be 600 (124 + 2×238)");
    }
}
