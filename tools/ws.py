import asyncio
import json
from websockets import client


URL = "ws://127.0.0.1:8080/schedule/updates"

    
async def main():
    def protocol_factory(*args, **kwargs) -> client.WebSocketClientProtocol:
        proto = client.WebSocketClientProtocol()
        proto.max_size = 2**48
        proto.read_limit = 2**48
        return proto
    
    error_printed = False
    
    while True:
        try:
            async with client.connect(
                uri=URL,
                create_protocol=protocol_factory
            ) as socket:
                error_printed = False
                i = -1
                async for message in socket:
                    print("new message")
                    i += 1
                    print("parsing...")
                    obj = json.loads(message)
                    print("writing...")
                    json.dump(
                        obj,
                        open(f"./debug/message-{i}.json", mode="w", encoding="utf8"),
                        ensure_ascii=False,
                        indent=2
                    )
                    print("wrote")
        except Exception as e:
            if not error_printed:
                print(f"error: {e}")
                error_printed = True
            await asyncio.sleep(1)
            continue


if __name__ == "__main__":
    asyncio.run(main())
