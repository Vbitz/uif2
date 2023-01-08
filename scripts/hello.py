#!/usr/bin/env python3

from websocket import create_connection
from json import dumps

ws = create_connection("ws://localhost:3012")


def send(obj):
    ws.send(dumps(obj))


ROOT = 0xff_ff_ff_ff

send({
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
