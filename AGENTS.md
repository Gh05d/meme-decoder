# WebAssembly + Rust + wasm-bindgen

1. **WebAssembly + Rust + wasm-bindgen**
    - The core is a Rust crate (`src/lib.rs`) compiled to WASM via `wasm-pack`.
    - Exports a handful of functions you can call from JS/TS (in the `pkg/` folder):
      - `parseBoopCreateToken`
      - `parsePumpFunCreate`
      - `parseCurveState`
      - `parseRaydiumInitialize`
      - `parseMoonshotTokenMint`

2. **Skipping Anchor’s 8-byte discriminator**
    - Anchor-built programs tag every instruction with an 8-byte “discriminator” (the first 8 bytes).  
      All parsers begin with:

      ```rust
      fn payload(data: &[u8]) -> Result<&[u8], JsValue> {
            if data.len() < 8 { Err(…) } else { Ok(&data[8..]) }
      }
      ```

    - That simply checks you have at least 8 bytes, then returns the slice after byte 8.

3. **Two parsing strategies**

    a) **Borsh-driven (Anchor-style)**
    - For Anchor programs whose IDL you have (Boop, Raydium Launchpad, Moonshot), you declare Rust structs/enums that exactly match the IDL layout and derive `BorshDeserialize`.
    - E.g., `CreateTokenBoopArgs`, `InitializeData` (which nests `MintParams`, `CurveParams`, `VestingParam`).
    - `.try_from_slice(payload)` deserializes the entire payload in one go.
    - You then pick off the fields you care about (e.g. name/symbol) into a slim serializable struct and return it via `serde_wasm_bindgen::to_value`.

    b) **Manual, byte-wise parsing**
    - For programs without a convenient Borsh IDL (Pump.fun / LetsBonk), you walk the buffer yourself.
    - Utility functions maintain an `offset: usize` and let you read:
      - `read_u32` / `read_u64`: 4 or 8 bytes little-endian → integer.
      - `read_string`: read a u32 length, then that many bytes, UTF-8 → `String`.
      - `read_pubkey`: read 32 bytes → Base58 (common Solana pubkey format).
    - In `parsePumpFunCreate` you do exactly that in sequence:
      1. name (length-prefixed string)
      2. symbol
      3. uri
      4. mint (32 bytes → Base58)
      5. bonding_curve (32 bytes → Base58)
      6. developer (32 bytes → Base58)
    - Collected into `ComputedTokenMetaData` and serialized out.

4. **Curve state & Moonshot**
    - `parseCurveState` also uses manual parsing to pull out 5 u64 reserves and a 1-byte boolean.
    - `parseMoonshotTokenMint` is basically a simpler manual string parse (name + symbol).

5. **JavaScript side**
    - Install / build the WASM:

      ```bash
      wasm-pack build --target web
      ```

    - In your JS/TS:

      ```javascript
      import init, {
         parsePumpFunCreate,
         parseRaydiumInitialize,
         /* … */
      } from 'meme-decoder';
      
      await init();
      
      const meta = parsePumpFunCreate(new Uint8Array(rawInstructionData));
      // meta.name, meta.symbol, meta.uri, meta.mint, meta.bondingCurve, meta.developer
      ```

6. **IDL reference**
    - The `idls/` directory holds the JSON IDLs (e.g., `launchlab.json`, `boop_idl.json`, `moonshot_idl.json`) that describe the on-chain layouts.
    - The Rust types in `lib.rs` are hand-matched to those IDLs.

---

## Bottom-line: Every exported function

1. Strips off the Anchor discriminator.
2. Either Borsh-deserializes into a Rust struct (for Anchor programs) or manually steps through the bytes (for Pump.fun/LetsBonk).
3. Re-packages just the fields you care about into a small serializable struct.
4. Hands it back to JS as a normal object you can inspect.
