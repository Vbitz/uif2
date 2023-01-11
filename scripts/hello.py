#!/usr/bin/env python3

import json
from websockets import connect
import asyncio


def label(s):
    return {
        "Label": {"text": s}
    }


def text_input():
    ev_id = "testing"
    return ev_id, {
        "TextInput": {"text": "", "on_changed": ev_id}
    }


def replace_node(object_id, node):
    return {
        "ReplaceNode": {
            "object_id": object_id,
            "node": node,
        }
    }


def append_child(parent_id, object_id, node):
    return {
        "AppendChild": {
            "parent_id": parent_id,
            "object_id": object_id,
            "node": node,
        }
    }


def transaction(client_id, *edits):
    return {
        "client_id": client_id,
        "edits": edits,
    }


async def main():
    async with connect("ws://127.0.0.1:3012/") as ws:
        async def send(obj):
            await ws.send(json.dumps(obj))

        ROOT = 0xff_ff_ff_ff

        ev_id, textbox = text_input()

        await send(transaction(
            "hello",
            append_child(ROOT, 10, label("Hello, World")),
            append_child(ROOT, 11, textbox),
        ))

        while True:
            ev = json.loads(await ws.recv())

            if "TextChanged" in ev:
                ev = ev["TextChanged"]

                if ev["id"] == ev_id:
                    await send(transaction(
                        "hello",
                        replace_node(10, label(ev["text"])),
                    ))

            print(ev)

asyncio.run(main())
