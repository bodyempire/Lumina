import asyncio

async def heartbeat():
    while True:
        print("Python is alive...")
        await asyncio.sleep(1)

async def logger():
    while True:
        print("--- 10s Python logger ---")
        await asyncio.sleep(10)

async def main():
    await asyncio.gather(heartbeat(), logger())

if __name__ == "__main__":
    asyncio.run(main())
