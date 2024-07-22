import pytest
import asyncio
from subxtpy import SubxtClient
@pytest.mark.asyncio
async def test_transfer_balance():
    # Test balance transfer from Alice to Bob
    client = await SubxtClient.new()

    # Bob's public key
    dest_public_key = bytes.fromhex('d43593c715fdd31c61141abd04a99fd6822c8558ed4bc1c7')

    # Transfer amount (adjust as necessary)
    amount = 10000

    # Perform the balance transfer
    tx_hash = await client.sign_and_submit()
    print("Transaction Hash: ", tx_hash)