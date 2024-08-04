import pytest
import asyncio
from subxtpy import SubxtClient

@pytest.mark.asyncio
async def test_fetch_account_nonce():
    client = await SubxtClient.new()

    # Alice's public key (in hex format)
    nonce = await client.runtime_api_call("AccountNonceApi", "account_nonce",
                                          ["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"])
    print("Account nonce: ", nonce)

    assert isinstance(nonce, int)
