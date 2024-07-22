import pytest
import asyncio
from subxtpy import SubxtClient

@pytest.mark.asyncio
async def test_fetch_constant():
    client = await SubxtClient.new()

    constant_value = await client.constant("System", "BlockLength")
    print("Constant value: ", constant_value)

    # Check that constant_value is a dictionary
    assert isinstance(constant_value, dict)

    # Check that the constant value contains expected keys (you should adjust these checks based on the actual structure of the constant)
    assert 'max' in constant_value

    mandatory = constant_value['max']['mandatory']
    assert isinstance(mandatory, int)
    assert mandatory >= 0