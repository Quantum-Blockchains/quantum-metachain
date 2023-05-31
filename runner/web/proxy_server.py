import asyncio
import time

import websockets
from common.logger import log
import common.config
import requests
import json

REMOTE_URL = "ws://localhost:9945"

async def hello(websocket, path):
    '''Called whenever a new connection is made to the server'''
    log.info("--------------------------------------------hello")
    while not ping_node():
        time.sleep(2)
    taskA = asyncio.create_task(clientToServer(websocket))
    await taskA


async def clientToServer(websocket):
    # async for message in ws:
    #     await websocket.send(message)
    async for message in websocket:
        print(message)
        response = request_node(message)
        print(response.text)
        await websocket.send(response.text)


async def serverToClient(ws, websocket):
    async for message in websocket:
        print(message)
        await ws.send(message)


def start():
    start_server = websockets.serve(hello,
                                    common.config.config_service.config.proxy_server_host,
                                    common.config.config_service.config.proxy_server_port)

    asyncio.get_event_loop().run_until_complete(start_server)
    asyncio.get_event_loop().run_forever()



def request_node(data):
    url = f"http://localhost:{common.config.config_service.config.node_http_rpc_port}"
    tmp = json.loads(data)
    response = requests.post(url, json=tmp)
    return response

def ping_node():
    url = f"http://localhost:{common.config.config_service.config.node_http_rpc_port}"
    data = {"id": 1, "jsonrpc": "2.0", "method": "system_peers", "params": []}
    try:
        # response = requests.post(url, json=data)
        requests.post(url, json=data)
    except Exception:
        return False
    return True