# rust_lua
rust写的给lua的websocket实现，自用，源于无法解决github的dns解析失败导致无法使用lua库的另类解决方案
```lua
-- 示例代码
local websocket = require("rust_websocket")
local client = websocket.connect("ws://127.0.0.1:8888/ws")
-- 发送消息
client:send("hello world")
client:poll_message(function (msg)
  print("获取一条消息")
  print(msg)
end)

-- 关闭连接
client:disconnect()
```

