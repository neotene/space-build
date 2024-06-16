using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;

using NativeWebSocket;

[Serializable]
public class TileBlock
{
    public int color;
}

[Serializable]
public class BlockData
{
    public String block_type;
    public String block_json;
    public Vector3 block_coords;
}

[Serializable]
public class Data
{
    public BlockData[] blocks;
}

public class Connection : MonoBehaviour
{
    WebSocket websocket;

    public GameObject TilePrefab;
    public GameObject TablePrefab;

    // Start is called before the first frame update
    async void Start()
    {
        websocket = new WebSocket("ws://localhost:2567");

        websocket.OnOpen += () =>
        {
            //   Debug.Log("Connection open!");
        };

        websocket.OnError += (e) =>
        {
            Debug.Log("Error! " + e);
        };

        websocket.OnClose += (e) =>
        {
            Debug.Log("Connection closed!");
        };

        websocket.OnMessage += (bytes) =>
        {
            //   Debug.Log("OnMessage!");
            //   Debug.Log(bytes);

            // getting the message as a string
            var message = System.Text.Encoding.UTF8.GetString(bytes);
            Debug.Log("OnMessage! " + message);

            Data data = new Data();

            data = JsonUtility.FromJson<Data>(message);

            foreach (BlockData blockData in data.blocks)
            {
                if (blockData.block_type == "tile")
                {
                    TileBlock tileBlock = new TileBlock();
                    tileBlock = JsonUtility.FromJson<TileBlock>(blockData.block_json);

                    Instantiate(TilePrefab, blockData.block_coords * 10, Quaternion.identity);
                }
            }

        };

        // Keep sending messages at every 0.3s
        // InvokeRepeating("SendWebSocketMessage", 0.0f, 0.3f);

        // waiting for messages
        await websocket.Connect();
    }

    void Update()
    {
#if !UNITY_WEBGL || UNITY_EDITOR
        websocket.DispatchMessageQueue();
#endif
    }

    //   async void SendWebSocketMessage()
    //   {
    //     if (websocket.State == WebSocketState.Open)
    //     {
    //       // Sending bytes
    //       await websocket.Send(new byte[] { 10, 20, 30 });

    //       // Sending plain text
    //       await websocket.SendText("plain text message");
    //     }
    //   }

    private async void OnApplicationQuit()
    {
        await websocket.Close();
    }

}