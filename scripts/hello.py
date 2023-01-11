#!/usr/bin/env python3

from websockets import connect
import asyncio
from json import dumps


def text_node(s):
    return {
        "TextNode": {"text": s}
    }


def append_child(parent_id, object_id, node):
    return {
        "parent_id": parent_id,
        "object_id": object_id,
        "node": node,
    }


def transaction(client_id, *edits):
    return {
        "client_id": client_id,
        "edits": edits,
    }


async def main():
    async with connect("ws://127.0.0.1:3012/") as ws:
        async def send(obj):
            await ws.send(dumps(obj))

        ROOT = 0xff_ff_ff_ff

        await send(transaction(
            "hello",
            append_child(ROOT, 10, text_node("Hello, World")),
        ))

asyncio.run(main())
