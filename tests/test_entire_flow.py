import asyncio
from subxtpy import SubxtClient, Keypair

async def main():
    client = await SubxtClient.from_url("ws://127.0.0.1:9944")

    alice_public_key = bytes.fromhex('d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d')

    # Read a storage value
    value = await client.storage("System", "Account", alice_public_key)
    print(value)

    # Fetch a constant
    constant_value = await client.constant("Balances", "ExistentialDeposit")
    print(constant_value)

    # Call a runtime API
    api_result = await client.runtime_api_call("AccountNonceApi", "account_nonce", alice_public_key)
    print(api_result)

    # Sign and submit a transaction
    from_keypair = Keypair.from_secret_key("e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a")

    transfer_payload = ["8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48", 1_000]
    transfer_tx_hash = await client.sign_and_submit(from_keypair, "Balances", "transfer_allow_death", transfer_payload)
    print("Transfer tx hash:", transfer_tx_hash)

asyncio.run(main())