import pytest
import asyncio
from subxtpy import SubxtClient

@pytest.mark.asyncio
async def test_subxt_client_creation():
    client = await SubxtClient.new()
    assert isinstance(client, SubxtClient)

@pytest.mark.asyncio
async def test_fetch_free_balance():
    client = await SubxtClient.new()

    # Alice's public key (in hex format)
    alice_public_key = bytes.fromhex('d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d')

    account_info = await client.storage("System", "Account", alice_public_key)
    # Check that free_balance is a dictionary
    assert isinstance(account_info, dict)

    # Check that 'data' and 'free' fields exist in the dictionary
    assert 'data' in account_info
    assert 'free' in account_info['data']

    # Check that the free balance is a number
    free_balance_value = account_info['data']['free']
    assert isinstance(free_balance_value, int)
    assert free_balance_value >= 0