# subxtpy &middot; ![build](https://github.com/paritytech/subxt/workflows/Rust/badge.svg) [![Documentation](https://docs.rs/subxt/badge.svg)](https://docs.rs/subxt)

**subxtpy** is a Python wrapper for the [subxt](https://github.com/paritytech/subxt) library. This library leverages the functionality provided by the `subxt` library to offer a convenient and efficient way to communicate with Substrate-based blockchains in Python.

## Features

| Feature                     | Description                                                                                          | Supported     |
|-----------------------------|------------------------------------------------------------------------------------------------------|---------------|
| Submit Extrinsics           | Submit transactions (extrinsics) to the blockchain.                                                  | ✅             |
| Read Storage Values         | Read and iterate over storage values on the blockchain.                                              | ✅             |
| Read Constants              | Fetch constants defined in the runtime metadata.                                                     | ✅             |
| Call Runtime APIs           | Call runtime APIs and retrieve their results.                                                        | ✅             |
| Dynamic Types               | Use dynamic types based on metadata for more flexible interactions.                                  | ✅             |
| Subscribe to Blocks, events | Subscribe to new blocks and read the extrinsics and events.                                          | ⏳ (Upcoming) |

## Usage

### Installation

The package will be published as soon as we have a stable version with the above feature set. For now, you can  build
the package locally by first installing `maturin` with:
```bash
pipx install maturin
```
and then running following command:
```bash
maturin develop
```

### Downloading Metadata from a Substrate Node
Use the `subxt-cli` tool to download the metadata for your target runtime from a node.

1. Install:

```bash
cargo install subxt-cli
```

2. Save the encoded metadata to a file:

```bash
subxt metadata -f bytes > artifacts/metadata.scale
```

This defaults to querying the metadata of a locally running node on the default `http://localhost:9933/`. If querying a different node, the `metadata` command accepts a `--url` argument.

### Example Usage

Here is an example of how to use `subxtpy` to interact with a Substrate-based blockchain:

```python
import asyncio
from subxtpy import SubxtClient, Keypair

async def main():
    client = await SubxtClient.from_url("ws://127.0.0.1:9944")

    alice_public_key = bytes.fromhex('d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d')

    # Read a storage value
    value = await client.storage("System", "Account", alice_public_key)
    print(value)

    # Fetch a constant
    constant_value = await client.constant("Balances", "ExistentialDeposit")
    print(constant_value)

    # Call a runtime API
    api_result = await client.runtime_api_call("AccountNonceApi", "account_nonce", alice_public_key)
    print(api_result)

    # Sign and submit a transaction
    from_keypair = Keypair.from_secret_key("e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a")

    transfer_payload = ["8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48", 1_000]
    transfer_tx_hash = await client.sign_and_submit(from_keypair, "Balances", "transfer_allow_death", transfer_payload)
    print("Transfer tx hash:", transfer_tx_hash)

asyncio.run(main())
```

## Subxtpy Documentation

For more details regarding utilizing `subxtpy`, please visit the [documentation](https://docs.rs/subxt/latest/subxt/).

## Testing
We wrote some tests by following the examples provided in the official [subxt repo](https://github.com/paritytech/subxt/tree/master/subxt/examples).
These tests can be run by running:

```bash
pytest
```

## Contributing

Contributions to `subxtpy` are welcome! If you encounter any issues or have suggestions for improvements, please open an issue or submit a pull request.

## Real World Usage

We will be providing guides for various real-world use cases here.


#### License

The entire code within this repository is licensed under the _Apache-2.0_ license. See [the LICENSE](./LICENSE.md) file for more details.