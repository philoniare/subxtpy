# BlockSubscription

A subscription to new blocks on the blockchain.

This class provides an asynchronous iterator over new blocks as they are finalized.

## Example Usage

```python
import asyncio
from subxtpy import SubxtClient

async def main():
    client = await SubxtClient.new()
    subscription = await client.subscribe_new_blocks()
    async for block in subscription:
        print(f"Block Number: {block['block_number']}")
        print(f"Block Hash: {block['block_hash']}")
        print("Extrinsics:")
        for extrinsic in block['extrinsics']:
            print(extrinsic)

asyncio.run(main())
```

## Methods

### `__aiter__()`

Return the asynchronous iterator object.

**Usage:**

```python
async for block in subscription:
    # process block
```

### `__anext__() -> dict`

Return the next block from the subscription.

**Yields:**

- `dict`: A dictionary containing the block number, block hash, and a list of extrinsics.

**Raises:**

- `StopAsyncIteration`: When no more blocks are available.