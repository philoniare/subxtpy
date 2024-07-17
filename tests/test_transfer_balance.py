import pytest
from subxtpy import OnlineClient

def test_constant_dynamic():
    api = OnlineClient()
    api.initialize()
    result = api.constants()
    height = api.block_length
    api.storagy_query()
    print("height: ", height)
    assert "success" in result.lower(), "Balance transfer failed!"
