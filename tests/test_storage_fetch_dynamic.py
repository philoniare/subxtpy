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

    account_info = await client.storage("System", "Account",
                                        ["d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"])
    assert isinstance(account_info, dict)

    assert 'data' in account_info
    assert 'free' in account_info['data']

    free_balance_value = account_info['data']['free']
    assert isinstance(free_balance_value, int)
    assert free_balance_value >= 0