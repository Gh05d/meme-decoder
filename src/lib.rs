use borsh::BorshDeserialize;
use bs58::encode as bs58_encode;
use js_sys::{BigInt, Object, Reflect};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use std::str;
use wasm_bindgen::prelude::*;

// Console logging macro
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format!($($t)*).into()));
}

// NOTE: Functions
/// Skip the 8-byte discriminator and return the payload or an error.
fn payload<'a>(data: &'a [u8]) -> Result<&'a [u8], JsValue> {
    if data.len() < 8 {
        Err(JsValue::from_str("Data too short"))
    } else {
        Ok(&data[8..])
    }
}

/// Read a little-endian integer of fixed byte length.
fn read_le<const N: usize>(buf: &[u8], off: &mut usize) -> Result<[u8; N], JsValue> {
    if buf.len() < *off + N {
        Err(JsValue::from_str("Unexpected buffer length"))
    } else {
        let mut arr = [0u8; N];
        arr.copy_from_slice(&buf[*off..*off + N]);
        *off += N;
        Ok(arr)
    }
}

/// Read a u32 in LE format.
fn read_u32(buf: &[u8], off: &mut usize) -> Result<u32, JsValue> {
    let bytes = read_le::<4>(buf, off)?;
    Ok(u32::from_le_bytes(bytes))
}

/// Read a u64 in LE format.
fn read_u64(buf: &[u8], off: &mut usize) -> Result<u64, JsValue> {
    let bytes = read_le::<8>(buf, off)?;
    Ok(u64::from_le_bytes(bytes))
}

/// Read a length-prefixed UTF-8 string.
fn read_string(buf: &[u8], off: &mut usize) -> Result<String, JsValue> {
    let len = read_u32(buf, off)? as usize;
    if buf.len() < *off + len {
        return Err(JsValue::from_str("String length exceeds buffer"));
    }
    let s =
        str::from_utf8(&buf[*off..*off + len]).map_err(|_| JsValue::from_str("Invalid UTF-8"))?;
    *off += len;
    Ok(s.to_owned())
}

/// Read a 32-byte public key and Base58-encode it.
fn read_pubkey(buf: &[u8], off: &mut usize) -> Result<String, JsValue> {
    let key = read_le::<32>(buf, off)?;
    Ok(bs58_encode(key).into_string())
}

// NOTE: Structs
#[derive(Serialize)]
struct InitializeSimple {
    name: String,
    symbol: String,
}

#[derive(BorshDeserialize, Debug)]
pub struct CreateTokenBoopArgs {
    pub salt: u64,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

/// Metadata struct for Pump.fun / LetsBonk create
#[derive(Serialize)]
struct ComputedTokenMetaData {
    name: String,
    symbol: String,
    uri: String,
    mint: String,
    #[serde(rename = "bondingCurve")]
    bonding_curve: String,
    developer: String,
}

// The three Curve variants
#[derive(BorshDeserialize, Serialize, Deserialize)]
pub struct ConstantCurve {
    pub supply: u64,
    pub total_base_sell: u64,
    pub total_quote_fund_raising: u64,
    pub migrate_type: u8,
}

#[derive(BorshDeserialize, Serialize, Deserialize)]
pub struct FixedCurve {
    pub supply: u64,
    pub total_quote_fund_raising: u64,
    pub migrate_type: u8,
}

#[derive(BorshDeserialize, Serialize, Deserialize)]
pub struct LinearCurve {
    pub supply: u64,
    pub total_quote_fund_raising: u64,
    pub migrate_type: u8,
}

// 3) CurveParams enum   matches IDL "CurveParams"
#[derive(BorshDeserialize, Serialize, Deserialize)]
pub enum CurveParams {
    Constant { data: ConstantCurve },
    Fixed { data: FixedCurve },
    Linear { data: LinearCurve },
}

#[derive(BorshDeserialize, Serialize, Deserialize)]
pub struct VestingParam {
    /// number of tokens locked, as a u64
    pub total_locked_amount: u64,
    /// cliff (in seconds, or whatever unit your IDL uses)
    pub cliff_period: u64,
    /// unlock period (same unit)
    pub unlock_period: u64,
}

// Struct matching the Anchor IDL for Raydium initialize instruction
#[derive(BorshDeserialize, Serialize, Deserialize)]
pub struct MintParams {
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(BorshDeserialize, Serialize, Deserialize)]
pub struct InitializeData {
    pub base_mint_param: MintParams,
    pub curve_param: CurveParams,
    pub vesting_param: VestingParam,
}

// NOTE: Parsers
/// WASM-exported parser for Boop.create_token
#[wasm_bindgen(js_name = "parseBoopCreateToken")]
pub fn parse_boop_create_token(data: &[u8]) -> Result<JsValue, JsValue> {
    let buf = payload(data)?;
    let args = CreateTokenBoopArgs::try_from_slice(buf)
        .map_err(|e| JsValue::from_str(&format!("Deserialization failed: {}", e)))?;

    let resp = InitializeSimple {
        name: args.name,
        symbol: args.symbol,
    };
    to_value(&resp).map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
}

/// WASM-exported parser for Pump.fun create instruction
#[wasm_bindgen(js_name = "parsePumpFunCreate")]
pub fn parse_pump_fun_create(data: &[u8]) -> Result<JsValue, JsValue> {
    let buf = payload(data)?;
    let mut off = 0;

    let name = read_string(buf, &mut off)?;
    let symbol = read_string(buf, &mut off)?;
    let uri = read_string(buf, &mut off)?;
    let mint = read_pubkey(buf, &mut off)?;
    let bonding_curve = read_pubkey(buf, &mut off)?;
    let developer = read_pubkey(buf, &mut off)?;

    let meta = ComputedTokenMetaData {
        name,
        symbol,
        uri,
        mint,
        bonding_curve,
        developer,
    };
    to_value(&meta).map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
}

/// WASM-exported parser for curve state using JS BigInt
#[wasm_bindgen(js_name = "parseCurveState")]
pub fn parse_curve_state_bigint(data: &[u8]) -> Result<JsValue, JsValue> {
    let buf = payload(data)?;
    let mut off = 0;

    // Read raw reserves as u64
    let virtual_token_reserves = read_u64(buf, &mut off)?;
    let virtual_sol_reserves = read_u64(buf, &mut off)?;
    let real_token_reserves = read_u64(buf, &mut off)?;
    let real_sol_reserves = read_u64(buf, &mut off)?;
    let token_total_supply = read_u64(buf, &mut off)?;

    // Read completion flag
    if buf.len() < off + 1 {
        return Err(JsValue::from_str("Unexpected end of buffer"));
    }
    let complete = buf[off] != 0;

    // Build a JS object with native BigInt fields
    let obj = Object::new();
    Reflect::set(
        &obj,
        &"virtual_token_reserves".into(),
        &BigInt::from(virtual_token_reserves).into(),
    )?;
    Reflect::set(
        &obj,
        &"virtual_sol_reserves".into(),
        &BigInt::from(virtual_sol_reserves).into(),
    )?;
    Reflect::set(
        &obj,
        &"real_token_reserves".into(),
        &BigInt::from(real_token_reserves).into(),
    )?;
    Reflect::set(
        &obj,
        &"real_sol_reserves".into(),
        &BigInt::from(real_sol_reserves).into(),
    )?;
    Reflect::set(
        &obj,
        &"token_total_supply".into(),
        &BigInt::from(token_total_supply).into(),
    )?;
    Reflect::set(&obj, &"complete".into(), &JsValue::from_bool(complete))?;

    Ok(JsValue::from(obj))
}

/// WASM-exported parser for Raydium initialize
#[wasm_bindgen(js_name = "parseRaydiumInitialize")]
pub fn parse_raydium_initialize(data: &[u8]) -> Result<JsValue, JsValue> {
    let buf: &[u8] = payload(data)?;
    // Reuse BorshDeserialize on your IDL-matching struct here.
    let init: InitializeData = InitializeData::try_from_slice(buf)
        .map_err(|e| JsValue::from_str(&format!("Deserialization failed: {}", e)))?;

    let simple = InitializeSimple {
        name: init.base_mint_param.name,
        symbol: init.base_mint_param.symbol,
    };
    to_value(&simple).map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
}

/// WASM-exported parser for Moonshot `initialize` instruction data
#[wasm_bindgen(js_name = "parseMoonshotTokenMint")]
pub fn parse_moonshot_token_mint(data: &[u8]) -> Result<JsValue, JsValue> {
    // 1. Get the payload (skip the 8-byte discriminator)
    let buf = payload(data)?;

    // First try the manual parser which is more reliable
    let mut off = 0;
    let name = match read_string(buf, &mut off) {
        Ok(name) => name,
        Err(_) => return Err(JsValue::from_str("Failed to parse name")),
    };

    let symbol = match read_string(buf, &mut off) {
        Ok(symbol) => symbol,
        Err(_) => return Err(JsValue::from_str("Failed to parse symbol")),
    };

    let token_info = InitializeSimple { name, symbol };

    // Convert to JsValue using to_value
    to_value(&token_info).map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
}
