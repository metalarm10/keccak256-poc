# keccak256-poc

Minimal CosmWasm contract that exposes a single read-only query: compute the
**original Keccak-256** (the Ethereum variant, NOT NIST SHA3-256) over an
arbitrary payload. Deployed on TX testnet so anyone can verify the chain
supports the algorithm without paying gas.

## What it proves

- A CosmWasm contract on TX testnet can compute Keccak-256 byte-exactly
  matching Ethereum/EVM behavior.
- The cryptographic primitive needed by Circle's xReserve attester signature
  flow (USDCx mint flow) is available via a contract-level dependency
  (`sha3` crate from RustCrypto), no chain modification required.

## Sanity vector

`keccak256(b"")` must equal
`c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470`.
If a reader gets `a7ffc6f8...` they are looking at NIST SHA3-256, not
Keccak-256. The unit test enforces this.

## Layout

```
keccak256-poc/
├── Cargo.toml          # cosmwasm-std 1.5, sha3 0.10, serde, thiserror
└── src/
    └── lib.rs           # instantiate (no-op) + Hash query + tests
```

~75 lines of Rust total. Stateless. No execute handlers.

## Build

```bash
# Local debug + tests (requires Rust 1.85+ for transitive edition-2024 deps)
cargo +1.85.0 test

# Optimized wasm artifact (deployable).
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.17.0

shasum -a 256 artifacts/keccak256_poc.wasm
```

## Deployed on TX testnet

| Field | Value |
|---|---|
| `code_id` | `3484` |
| Contract address | `testcore14sqgkp6s9m7k4dyarf7krkkra7vg2z834avkpg3ar9tg6t676cvsxapdvq` |
| wasm checksum | `189e93a6d88581b05de39e314cc7b115df435ee23f65a2ba0d90091a36470c19` |
| Store tx | `B3F4CFA35A7DB2C9C7A4E3F77BD4DFA909ADA8B27D3BA2649747DA43BF219F4C` |
| Instantiate tx | `441DCAC81E188AB66606B302BFEACDD4A084E5660195B133379CDA55EC4C1E71` |

## Verification queries (anyone can run, no gas, no signing)

### Setup — one-time env exports

Adjust `TXD` to your local binary path. The `NODE` and `CHAIN` are testnet
endpoints anyone can use without an account.

```bash
export TXD=/path/to/txd
export CHAIN="TX-testnet-1"
export NODE="https://full-node-pluto.testnet-1.TX.dev:26657"
export CONTRACT="testcore14sqgkp6s9m7k4dyarf7krkkra7vg2z834avkpg3ar9tg6t676cvsxapdvq"
```

### Vector 1 — empty bytes (algorithm sanity check)

```bash
$TXD --chain-id=$CHAIN --node=$NODE q wasm contract-state smart \
  $CONTRACT '{"hash":{"payload":""}}'
```

Expected response:

```yaml
data:
  digest: xdJGAYb3IzySfn2y3McDwOUAtlPKgic7e/rYBF2FpHA=
```

That base64 decodes to
`c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470` — the
canonical Ethereum keccak256 of empty bytes. If the contract were using
NIST SHA3-256 instead, you'd get `a7ffc6f8...`, and the response base64
would start with `p///...`.

To verify the digest matches the canonical value yourself:

```bash
# base64-decode the response and hex-encode
printf 'xdJGAYb3IzySfn2y3McDwOUAtlPKgic7e/rYBF2FpHA=' | base64 -d | xxd -p -c 64
# expect: c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
```

### Vector 2 — hash arbitrary bytes (`b"hello world"`)

```bash
PAYLOAD_B64=$(printf 'hello world' | base64)
$TXD --chain-id=$CHAIN --node=$NODE q wasm contract-state smart \
  $CONTRACT "{\"hash\":{\"payload\":\"$PAYLOAD_B64\"}}"
```

Expected response:

```yaml
data:
  digest: RxcyhajXNB5ely/GdyhjhPgC+O9CpexfA7v6JUywH60=
```

Decodes to `47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad` —
the well-known Ethereum keccak256 of `"hello world"`.

### Vector 3 — hash any payload you want

```bash
# Replace 'your bytes here' with anything
PAYLOAD_B64=$(printf 'your bytes here' | base64)
$TXD --chain-id=$CHAIN --node=$NODE q wasm contract-state smart \
  $CONTRACT "{\"hash\":{\"payload\":\"$PAYLOAD_B64\"}}"
```

Compare against any independent keccak256 implementation
(Etherscan's hash tool, Python's `pycryptodome`, Node's `js-sha3`, etc.) —
results must match byte-for-byte.

### Alternative: query via REST gateway (no binary needed)

The same query as a plain HTTP call:

```bash
# Base64 of {"hash":{"payload":""}} → eyJoYXNoIjp7InBheWxvYWQiOiIifX0=
curl -s "https://full-node-pluto.testnet-1.TX.dev:1317/cosmwasm/wasm/v1/contract/$CONTRACT/smart/eyJoYXNoIjp7InBheWxvYWQiOiIifX0="
```

Returns `{"data":{"digest":"xdJGAYb3IzySfn2y3McDwOUAtlPKgic7e/rYBF2FpHA="}}`.
Useful for browser-based verification without installing a chain binary.
