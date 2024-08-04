import pytest
import asyncio
from subxtpy import SubxtClient

@pytest.mark.asyncio
async def test_fetch_events():
    client = await SubxtClient.new()
    events = await client.events()

    assert isinstance(events, list)
    for event in events:
        assert isinstance(event, dict)
        assert 'pallet' in event
        assert 'variant' in event
        assert 'fields' in event
