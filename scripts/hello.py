#!/usr/bin/env python3

import json
from websockets import connect
import asyncio
import os


def random_id():
    return os.urandom(4).hex()


def window(title):
    return {
        "Window": {"title": title}
    }


def left_to_right_layout():
    return {
        "LeftToRightLayout": {}
    }


def label(s):
    return {
        "Label": {"text": s}
    }


def text_input():
    ev_id = random_id()
    return ev_id, {
        "TextInput": {"text": "", "on_changed": ev_id}
    }


def button(text):
    ev_id = random_id()
    return ev_id, {
        "Button": {"text": text, "on_clicked": ev_id}
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
        btn_id, btn = button("Testing")

        await send(transaction(
            "hello",
            append_child(ROOT, 10, window("Hello")),
            append_child(10, 11, left_to_right_layout()),
            append_child(11, 12, label("Hello, World")),
            append_child(11, 13, textbox),
            append_child(10, 14, btn),
        ))

        while True:
            ev = json.loads(await ws.recv())

            if "TextChanged" in ev:
                ev = ev["TextChanged"]

                if ev["id"] == ev_id:
                    await send(transaction(
                        "hello",
                        replace_node(12, label(ev["text"])),
                    ))
            elif "Clicked" in ev:
                ev = ev["Clicked"]
                if ev["id"] == btn_id:
                    await send(transaction(
                        "hello",
                        replace_node(
                            12, label("The button has been clicked.")),
                    ))

            print(ev)

asyncio.run(main())
