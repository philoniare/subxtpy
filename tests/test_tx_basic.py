import pytest
from subxtpy import SubxtClient, Keypair

@pytest.mark.asyncio
async def test_transfer_balance():
    # Test balance transfer from Alice to Bob
    client = await SubxtClient.new()

    # Bob's public key
    from_keypair = Keypair.from_secret_key("e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a")

    # Perform the balance transfer
    transfer_payload = ["8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48", 1_000]
    tx_hash = await client.sign_and_submit(from_keypair, "Balances", "transfer_allow_death", transfer_payload)
    print("Transaction Hash: ", tx_hash)