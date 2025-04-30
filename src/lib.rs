use borsh::BorshDeserialize;
use bs58::encode as bs58_encode;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use std::str;
use wasm_bindgen::prelude::*;

// NOTE: Data Structures
// Metadata for a newly created token
#[derive(Serialize, Deserialize)]
pub struct ComputedTokenMetaData {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub mint: String,
    pub bondingCurve: String,
    pub developer: String,
}

// Struct matching the Anchor IDL for Raydium initialize instruction
#[derive(BorshDeserialize, Serialize, Deserialize)]
pub struct BaseMintParam {
    pub name: String,
    pub symbol: String,
}

#[derive(BorshDeserialize)]
pub struct InitializeData {
    pub base_mint_param: BaseMintParam,
}

// Struct for bonding curve data
#[derive(Serialize, Deserialize)]
pub struct BondingCurveState {
    pub virtual_token_reserves: u64,
    pub virtual_sol_reserves: u64,
    pub real_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub token_total_supply: u64,
    pub complete: bool,
}

// NOTE: Raydium Borsh Decode
/// Decode a Raydium Launchpad "initialize" instruction payload via Borsh
#[wasm_bindgen]
pub fn parseRaydiumInitialize(buf: &[u8]) -> JsValue {
    match InitializeData::try_from_slice(buf) {
        Ok(data) => {
            // Return only name and symbol
            to_value(&data.base_mint_param).unwrap_or(JsValue::NULL)
        }
        Err(_) => JsValue::NULL,
    }
}

// === Pump.fun / LetsBonk Instruction Parser ===
/// Internal parser returning Option; uses no `?` in JsValue function
fn try_parse_create(data: &[u8]) -> Option<ComputedTokenMetaData> {
    if data.len() < 8 {
        return None;
    }
    let mut offset = 8;
    let mut meta = ComputedTokenMetaData {
        name: String::new(),
        symbol: String::new(),
        uri: String::new(),
        mint: String::new(),
        bondingCurve: String::new(),
        developer: String::new(),
    };

    // Helper to read a little-endian u32
    fn read_u32_le(buf: &[u8], off: &mut usize) -> Option<u32> {
        if buf.len() < *off + 4 {
            return None;
        }
        let val = u32::from_le_bytes([buf[*off], buf[*off + 1], buf[*off + 2], buf[*off + 3]]);
        *off += 4;
        Some(val)
    }

    // Read a UTF-8 string field
    fn read_string(buf: &[u8], off: &mut usize) -> Option<String> {
        let len = read_u32_le(buf, off)? as usize;
        if buf.len() < *off + len {
            return None;
        }
        let s = str::from_utf8(&buf[*off..*off + len]).ok()?;
        *off += len;
        Some(s.to_string())
    }

    // Read a 32-byte publicKey and Base58-encode it
    fn read_pubkey(buf: &[u8], off: &mut usize) -> Option<String> {
        if buf.len() < *off + 32 {
            return None;
        }
        let key = &buf[*off..*off + 32];
        *off += 32;
        Some(bs58_encode(key).into_string())
    }

    // Parse fields in order
    meta.name = read_string(data, &mut offset)?;
    meta.symbol = read_string(data, &mut offset)?;
    meta.uri = read_string(data, &mut offset)?;
    meta.mint = read_pubkey(data, &mut offset)?;
    meta.bondingCurve = read_pubkey(data, &mut offset)?;
    meta.developer = read_pubkey(data, &mut offset)?;

    Some(meta)
}

// WASM-exported parser that wraps the Option into JsValue
#[wasm_bindgen]
pub fn parsePumpFunCreate(data: &[u8]) -> JsValue {
    if let Some(meta) = try_parse_create(data) {
        to_value(&meta).unwrap_or(JsValue::NULL)
    } else {
        JsValue::NULL
    }
}

// NOTE: Bonding-Curve State Decoder
fn try_parse_curve(data: &[u8]) -> Option<BondingCurveState> {
    // Expect at least 8(discriminator) + 6*8(u64) + 1(bool) = 57 bytes
    if data.len() < 8 + 6 * 8 + 1 {
        return None;
    }
    let mut off = 8; // skip discriminator

    fn read_u64_le(buf: &[u8], off: &mut usize) -> Option<u64> {
        if buf.len() < *off + 8 {
            return None;
        }
        let v = u64::from_le_bytes([
            buf[*off],
            buf[*off + 1],
            buf[*off + 2],
            buf[*off + 3],
            buf[*off + 4],
            buf[*off + 5],
            buf[*off + 6],
            buf[*off + 7],
        ]);
        *off += 8;
        Some(v)
    }

    let virtual_token_reserves = read_u64_le(data, &mut off)?;
    let virtual_sol_reserves = read_u64_le(data, &mut off)?;
    let real_token_reserves = read_u64_le(data, &mut off)?;
    let real_sol_reserves = read_u64_le(data, &mut off)?;
    let token_total_supply = read_u64_le(data, &mut off)?;

    // Read the `complete` flag (1 byte, non-zero = true)
    let complete = data.get(off).copied().map(|b| b != 0)?;

    Some(BondingCurveState {
        virtual_token_reserves,
        virtual_sol_reserves,
        real_token_reserves,
        real_sol_reserves,
        token_total_supply,
        complete,
    })
}

#[wasm_bindgen]
pub fn parse_curve_state(data: &[u8]) -> JsValue {
    if let Some(state) = try_parse_curve(data) {
        to_value(&state).unwrap_or(JsValue::NULL)
    } else {
        JsValue::NULL
    }
}
