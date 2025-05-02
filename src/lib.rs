use borsh::BorshDeserialize;
use bs58::decode as bs58_decode;
use bs58::encode as bs58_encode;
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
    bondingCurve: String,
    developer: String,
}

/// Bonding curve state struct.
#[derive(Serialize)]
struct BondingCurveState {
    virtual_token_reserves: u64,
    virtual_sol_reserves: u64,
    real_token_reserves: u64,
    real_sol_reserves: u64,
    token_total_supply: u64,
    complete: bool,
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

// 3) CurveParams enum ⟶ matches IDL "CurveParams"
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

// Your Moonshot struct matching the IDL layout
#[derive(BorshDeserialize, Serialize)]
pub struct TokenMintParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
    pub collateral_currency: u8,
    pub amount: u64,
    pub curve_type: u8,
    pub migration_target: u8,
}

// NOTE: Parsers
/// WASM-exported parser for Boop.create_token
#[wasm_bindgen]
pub fn parseBoopCreateToken(data: &[u8]) -> Result<JsValue, JsValue> {
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
#[wasm_bindgen]
pub fn parsePumpFunCreate(data: &[u8]) -> Result<JsValue, JsValue> {
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
        bondingCurve: bonding_curve,
        developer,
    };
    to_value(&meta).map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
}

/// WASM-exported parser for curve state
#[wasm_bindgen]
pub fn parse_curve_state(data: &[u8]) -> Result<JsValue, JsValue> {
    let buf = payload(data)?;
    let mut off = 0;
    let virtual_token_reserves = read_u64(buf, &mut off)?;
    let virtual_sol_reserves = read_u64(buf, &mut off)?;
    let real_token_reserves = read_u64(buf, &mut off)?;
    let real_sol_reserves = read_u64(buf, &mut off)?;
    let token_total_supply = read_u64(buf, &mut off)?;
    if buf.len() < off + 1 {
        return Err(JsValue::from_str("Unexpected end of buffer"));
    }
    let complete = buf[off] != 0;

    let state = BondingCurveState {
        virtual_token_reserves,
        virtual_sol_reserves,
        real_token_reserves,
        real_sol_reserves,
        token_total_supply,
        complete,
    };
    to_value(&state).map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
}

/// WASM-exported parser for Raydium initialize
#[wasm_bindgen]
pub fn parseRaydiumInitialize(data: &[u8]) -> Result<JsValue, JsValue> {
    let buf = payload(data)?;
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
#[wasm_bindgen]
pub fn parseMoonshotTokenMint(ix_data: &str) -> Result<JsValue, JsValue> {
    // 1. Decode base58 string to raw bytes
    let raw = bs58_decode(ix_data)
        .into_vec()
        .map_err(|e| JsValue::from_str(&format!("Base58 decode failed: {}", e)))?;

    // 2. Strip the 8-byte Anchor discriminator
    let buf = payload(&raw)?;

    // 3. Deserialize into your struct
    let params = TokenMintParams::try_from_slice(buf)
        .map_err(|e| JsValue::from_str(&format!("Deserialization failed: {}", e)))?;

    // 4. Convert Rust struct into a JS value
    to_value(&params).map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
}
