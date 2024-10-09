import pytest
import asyncio
from subxtpy import SubxtClient

@pytest.mark.asyncio
async def test_subscribe_new_blocks():
    client = await SubxtClient.new()
    subscription = await client.subscribe_new_blocks()

    # Check that subscription is an asynchronous iterator
    assert hasattr(subscription, '__aiter__')
    assert hasattr(subscription, '__anext__')

    # Fetch a single block from the subscription for testing
    async for block in subscription:
        print(block)
        assert isinstance(block, dict)
        assert 'block_number' in block
        assert 'block_hash' in block
        assert 'extrinsics' in block

        # Check that 'extrinsics' is a list
        assert isinstance(block['extrinsics'], list)
        # Optionally, perform more detailed checks on 'extrinsics'
        for extrinsic in block['extrinsics']:
            assert isinstance(extrinsic, dict)
            assert 'pallet' in extrinsic
            assert 'call' in extrinsic
            assert 'fields' in extrinsic
            assert 'signed_extensions' in extrinsic
        break  # Stop after the first block to prevent infinite loop