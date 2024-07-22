import pytest
import asyncio
from subxtpy import SubxtClient

@pytest.mark.asyncio
async def test_fetch_events():
    client = await SubxtClient.new()

    events = await client.get_events()
    print("Events: ", events)

    # Check that events is a list
    assert isinstance(events, list)

    # Check that each event is a dictionary and contains expected keys
    for event in events:
        assert isinstance(event, dict)
        assert 'pallet' in event
        assert 'variant' in event
        assert 'fields' in event
