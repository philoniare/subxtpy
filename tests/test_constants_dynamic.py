import pytest
import asyncio
from subxtpy import SubxtClient

@pytest.mark.asyncio
async def test_fetch_constant():
    client = await SubxtClient.new()

    constant_value = await client.constant("System", "BlockLength")
    print("Constant value: ", constant_value)

    assert isinstance(constant_value, dict)
    assert 'max' in constant_value

    mandatory = constant_value['max']['mandatory']
    assert isinstance(mandatory, int)
    assert mandatory >= 0