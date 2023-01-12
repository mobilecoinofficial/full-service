## Full Service Python API ( soon ™️  to be CLI as well )

To get started, you can do 'poetry install' and 'poetry shell' to set up an enviornment. 
More simply, you can just 'pip install aiohttp' and you're ready to go, as long as you're using Python 3.8+

Here is a simple example of usage: 

```python
from fullservice import FullServiceAPIv2 as v2 
import asyncio 

async def main():
    fs = v2()
    existing_accounts = await fs.get_accounts()
    print(existing_accounts)

if __name__ == '__main__':
    asyncio.run(main())
```
