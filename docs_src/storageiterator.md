# StorageIterator

An asynchronous iterator over storage key-value pairs.

This class allows you to iterate over storage entries retrieved from the blockchain.

## Example Usage

```python
import asyncio
from subxtpy import SubxtClient

async def main():
    client = await SubxtClient.new()
    iterator = await client.storage_iter("System", "Account", b"")
    async for item in iterator:
        print(f"Key Bytes: {item['key_bytes']}")
        print(f"Keys: {item['keys']}")
        print(f"Value: {item['value']}")

asyncio.run(main())
```

## Methods

### `__aiter__()`

Return the asynchronous iterator object.

**Usage:**

```python
async for item in iterator:
    # process item
```

### `__anext__() -> dict`

Return the next storage key-value pair from the iterator.

**Yields:**

- `dict`: A dictionary containing the key bytes, keys, and value.

**Raises:**

- `StopAsyncIteration`: When no more items are available.