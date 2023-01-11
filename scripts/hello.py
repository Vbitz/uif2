#!/usr/bin/env python3

from websockets import connect
import asyncio
from json import dumps


async def main():
    async with connect("ws://127.0.0.1:3012/") as ws:
        async def send(obj):
            await ws.send(dumps(obj))

        ROOT = 0xff_ff_ff_ff

        await send({
            "client_id": "hello",
            "edits": [
                {
                    "AppendChild": {
                        "parent_id": ROOT,
                        "object_id": 10,
                        "node": {
                            "TextNode": {
                                "text": "Hello, World",
                            }
                        }
                    },
                }
            ]
        })

asyncio.run(main())
