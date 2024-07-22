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

    free_balance = await client.dynamic("System", "Account", alice_public_key)
    print("Free balance: ", free_balance)

    # Check that free_balance is a dictionary
    assert isinstance(free_balance, dict)

    # Check that 'data' and 'free' fields exist in the dictionary
    assert 'data' in free_balance
    assert 'free' in free_balance['data']

    # Check that the free balance is a number
    free_balance_value = free_balance['data']['free']
    print("Free: ", free_balance_value)
    assert isinstance(free_balance_value, int)
    assert free_balance_value >= 0