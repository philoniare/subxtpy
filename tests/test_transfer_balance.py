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

    free_balance = await client.fetch_free_balance(alice_public_key)
    print("Free: ", free_balance)
    assert isinstance(free_balance, str)
    assert free_balance.isdigit()  # Assuming the balance is returned as a string of digits
