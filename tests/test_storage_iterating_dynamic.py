import pytest
import asyncio
from subxtpy import SubxtClient

@pytest.mark.asyncio
async def test_fetch_storage_entries():
    client = await SubxtClient.new()
    alice_public_key = bytes.fromhex('d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d')
    gen = await client.storage_iter("System", "Account", alice_public_key)
    async for result in gen:
        assert isinstance(result, dict)
        assert 'key_bytes' in result
        assert 'keys' in result
        assert 'value' in result