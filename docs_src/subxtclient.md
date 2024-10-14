# SubxtClient

A client for interacting with a Substrate-based blockchain.

This class provides methods to interact with the blockchain, including fetching storage entries, constants, events, making runtime API calls, and submitting transactions.

## Class Methods

### `SubxtClient.new() -> SubxtClient`

Create a new `SubxtClient` instance asynchronously.

**Returns:**

- `SubxtClient`: A new client connected to the default network.

**Raises:**

- `RuntimeError`: If the client fails to connect.

**Example:**

```python
import asyncio
from subxtpy import SubxtClient

async def main():
    client = await SubxtClient.new()
    # Use the client for further operations

asyncio.run(main())
```

## Instance Methods

### `storage(pallet_name: str, entry_name: str, key: list) -> Any`

Fetch a storage entry from the blockchain asynchronously.

**Parameters:**

- `pallet_name` (str): The name of the pallet.
- `entry_name` (str): The name of the storage entry.
- `key` (list): A list of keys for the storage entry.

**Returns:**

- `Any`: The value of the storage entry.

**Raises:**

- `RuntimeError`: If fetching the storage entry fails.
- `ValueError`: If the storage entry is not found.

**Example:**

```python
import asyncio
from subxtpy import SubxtClient

async def main():
    client = await SubxtClient.new()
    balance = await client.storage("Balances", "FreeBalance", [account_id])
    print(f"Balance: {balance}")

asyncio.run(main())
```

### `constant(pallet_name: str, constant_name: str) -> Any`

Fetch a constant value from the blockchain asynchronously.

**Parameters:**

- `pallet_name` (str): The name of the pallet.
- `constant_name` (str): The name of the constant.

**Returns:**

- `Any`: The value of the constant.

**Raises:**

- `RuntimeError`: If fetching the constant fails.

**Example:**

```python
import asyncio
from subxtpy import SubxtClient

async def main():
    client = await SubxtClient.new()
    existential_deposit = await client.constant("Balances", "ExistentialDeposit")
    print(f"Existential Deposit: {existential_deposit}")

asyncio.run(main())
```

### `events() -> list`

Fetch events from the blockchain asynchronously.

**Returns:**

- `list`: A list of events, where each event is a dictionary containing pallet, variant, and fields.

**Raises:**

- `RuntimeError`: If fetching events fails.

**Example:**

```python
import asyncio
from subxtpy import SubxtClient

async def main():
    client = await SubxtClient.new()
    events = await client.events()
    for event in events:
        print(event)

asyncio.run(main())
```

### `runtime_api_call(pallet_name: str, entry_name: str, key: list) -> Any`

Perform a runtime API call to the blockchain asynchronously.

**Parameters:**

- `pallet_name` (str): The name of the pallet.
- `entry_name` (str): The name of the runtime API function.
- `key` (list): A list of arguments for the runtime API call.

**Returns:**

- `Any`: The result of the runtime API call.

**Raises:**

- `RuntimeError`: If the runtime API call fails.

**Example:**

```python
import asyncio
from subxtpy import SubxtClient

async def main():
    client = await SubxtClient.new()
    result = await client.runtime_api_call("SomePallet", "someFunction", [arg1, arg2])
    print(f"Result: {result}")

asyncio.run(main())
```

### `storage_iter(pallet_name: str, entry_name: str, key: bytes) -> StorageIterator`

Iterate over storage entries from the blockchain asynchronously.

**Parameters:**

- `pallet_name` (str): The name of the pallet.
- `entry_name` (str): The name of the storage entry.
- `key` (bytes): The prefix key for iteration.

**Returns:**

- `StorageIterator`: An asynchronous iterator over storage key-value pairs.

**Raises:**

- `RuntimeError`: If the iteration fails.

**Example:**

```python
import asyncio
from subxtpy import SubxtClient

async def main():
    client = await SubxtClient.new()
    iterator = await client.storage_iter("System", "Account", b"")
    async for item in iterator:
        print(item)

asyncio.run(main())
```

### `sign_and_submit(from: Keypair, pallet_name: str, entry_name: str, payload: list) -> str`

Sign and submit a transaction to the blockchain asynchronously.

**Parameters:**

- `from` (Keypair): The keypair to sign the transaction.
- `pallet_name` (str): The name of the pallet.
- `entry_name` (str): The name of the extrinsic.
- `payload` (list): A list of arguments for the extrinsic.

**Returns:**

- `str`: The transaction hash as a hexadecimal string.

**Raises:**

- `RuntimeError`: If signing or submitting the transaction fails.

**Example:**

```python
import asyncio
from subxtpy import SubxtClient, Keypair

async def main():
    client = await SubxtClient.new()
    keypair = Keypair.from_secret_key("your_64_character_hex_secret_key")
    tx_hash = await client.sign_and_submit(keypair, "Balances", "transfer", [dest_account_id, amount])
    print(f"Transaction Hash: {tx_hash}")

asyncio.run(main())
```

### `subscribe_new_blocks() -> BlockSubscription`

Subscribe to new blocks on the blockchain asynchronously.

**Returns:**

- `BlockSubscription`: An asynchronous iterator that yields blocks as they are finalized.

**Raises:**

- `RuntimeError`: If the subscription fails.

**Example:**

```python
import asyncio
from subxtpy import SubxtClient

async def main():
    client = await SubxtClient.new()
    subscription = await client.subscribe_new_blocks()
    async for block in subscription:
        print(block)

asyncio.run(main())
```