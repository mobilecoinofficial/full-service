import asyncio
import pytest

# In order to import just one wallet for the whole test session, we need to set the
# asyncio-pytest event loop to session scope.
@pytest.fixture(scope="session")
def event_loop():
    policy = asyncio.get_event_loop_policy()
    loop = policy.new_event_loop()
    yield loop
    loop.close()
