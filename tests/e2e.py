import asyncio
from subxtpy import SubxtClient, Keypair

async def main():
    client = await SubxtClient.from_url("ws://127.0.0.1:9944")

    # Read a storage value
    value = await client.storage("System", "Account",
                                 ["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"])
    print(value)

    # Fetch a constant
    constant_value = await client.constant("Balances", "ExistentialDeposit")
    print(constant_value)

    # Call a runtime API
    api_result = await client.runtime_api_call("AccountNonceApi", "account_nonce",
                                               ["e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"])
    print(api_result)

    # Sign and submit a transaction
    from_keypair = Keypair.from_secret_key("e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a")

    remark_payload = ["Hello"]
    transfer_tx_hash = await client.sign_and_submit(from_keypair, "System", "remark", remark_payload)
    print("Remark tx hash:", transfer_tx_hash)

asyncio.run(main())