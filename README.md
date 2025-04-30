# Meme Decoder

A WebAssembly (WASM) library for decoding Solana meme token creation instructions from various platforms.

## Overview

Meme Decoder is a Rust library compiled to WebAssembly that provides utilities for parsing and decoding token creation instructions on Solana. It currently supports:

- Raydium Launchpad "initialize" instruction payloads
- Pump.fun / LetsBonk token creation instructions

The library extracts key metadata from these instructions, including token name, symbol, URI, mint address, bonding curve, and developer information.

## Features

Decode Raydium Launchpad initialization data using Borsh deserialization
Parse Pump.fun and LetsBonk token creation instructions
Extract token metadata from raw instruction data
WebAssembly compatibility for use in JavaScript/TypeScript applications
Installation

## Clone the repository

```bash
git clone <https://github.com/yourusername/meme-decoder.git>
cd meme-decoder
```

## Build the WebAssembly package

`wasm-pack build --target web`

## Usage

In JavaScript/TypeScript

```typescript
import init, { parse_create_instruction, decode_initialize } from 'meme-decoder';

// Initialize the WASM module
await init();

// Parse a Pump.fun/LetsBonk token creation instruction
const instructionData = new Uint8Array([/*your instruction data*/]);
const tokenMetadata = parse_create_instruction(instructionData);
if (tokenMetadata) {
  console.log('Token Name:', tokenMetadata.name);
  console.log('Token Symbol:', tokenMetadata.symbol);
  console.log('Token URI:', tokenMetadata.uri);
  console.log('Mint Address:', tokenMetadata.mint);
  console.log('Bonding Curve:', tokenMetadata.bonding_curve);
  console.log('Developer:', tokenMetadata.developer);
}

// Decode a Raydium Launchpad initialize instruction
const raydiumData = new Uint8Array([/*your Raydium instruction data*/]);
const raydiumMetadata = decode_initialize(raydiumData);
if (raydiumMetadata) {
  console.log('Token Name:', raydiumMetadata.name);
  console.log('Token Symbol:', raydiumMetadata.symbol);
}
```

## API Reference

`decode_initialize(buf: Uint8Array) → Object | null`
Decodes a Raydium Launchpad "initialize" instruction payload using Borsh deserialization.

*Parameters:*

- `buf`: The raw instruction data as a Uint8Array

*Returns:*

- An object containing name and symbol if successful, or null if parsing fails

`parse_create_instruction(data: Uint8Array) → Object | null`
Parses a Pump.fun or LetsBonk token creation instruction.

*Parameters:*

- `data`: The raw instruction data as a Uint8Array

*Returns:*

- A ComputedTokenMetaData object if successful, or null if parsing fails
