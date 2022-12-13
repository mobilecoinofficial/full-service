## Full Service Python API ( soon ™️  to be CLI as well )

To get started, you can do 'poetry install --no-dev' and 'poetry shell' to set up an enviornment. 

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
