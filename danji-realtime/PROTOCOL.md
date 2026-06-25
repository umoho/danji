# danji-realtime Socket 协议

## 概述

danji-realtime daemon 通过 Unix domain socket (`/tmp/danji.sock`) 接受外部控制。
客户端连接后，发送一行文本命令，daemon 返回一行文本响应。

## 传输

- **类型**：`AF_UNIX` / `SOCK_STREAM`
- **路径**：`/tmp/danji.sock`
- **编码**：UTF-8
- **分隔符**：`\n`（LF）
- **最大命令长度**：4096 字节（协议层未强制限制，由实现保证）

## 命令总表

| 命令 | 参数 | 响应 |
|------|------|------|
| `bypass` | `on` / `off` | `OK` |
| `volume` | `<dB>` | `OK` |
| `gain` | `<dB>` | `OK` |
| `bplus` | `<V>` | `OK` |
| `tube` | `<type>` | `OK` |
| `model` | `<name>` | `OK` |
| `status` | — | 参数表 |
| `stop` | — | — |

## 命令详情

### bypass

绕过音频处理，输入直通输出。

```
bypass on      # 启用 bypass（直通）
bypass off     # 禁用 bypass（电子管处理）
```

- 默认值：`off`
- 实现：原子 bool 切换，音频 callback 下一采样立即生效
- 响应：`OK`

### volume

输出音量。系数 `10^(dB/20)` 作用于处理后信号。

```
volume -12     # -12 dB
volume 0       # 0 dB（最大输出，小心）
volume -60     # -60 dB（最低）
```

- 默认值：`-12`
- 范围：`-60` ~ `+12`
- 实现：原子 float，音频 callback 下一采样立即生效
- 响应：`OK`

### gain

输入增益。系数 `10^(dB/20)` 作用于输入信号，控制电子管失真程度。

```
gain -38       # -38 dB（轻度失真）
gain 0         # 0 dB（中等失真）
gain 10        # +10 dB（重度失真）
```

- 默认值：`-38`
- 范围：`-96` ~ `+10`
- 实现：原子 float，音频 callback 下一采样立即生效
- 响应：`OK`

### bplus

B+ 屏极电压。改变电子管工作点。

```
bplus 300      # 300 V（典型值）
bplus 150      # 150 V（低屏压，clean）
bplus 450      # 450 V（高屏压，headroom 更大）
```

- 默认值：`300`
- 范围：`50` ~ `600`
- 单位：伏特
- 实现：原子 float，下个 `process_sample` 调用 `set_bplus()` 生效
- 响应：`OK`

### tube

切换电子管型号。预热 3000 采样后热替换引擎中的 Simulator。

```
tube 12AX7     # 高增益前级管
tube 12AU7     # 中增益前级管
tube 12AT7     # 中高增益前级管
tube 6DJ8      # 低增益前级管
tube 6L6GC     # 束射四极管（三极接法）
tube 6550      # 功率五极管（三极接法）
tube EL34      # 功率五极管（三极接法）
tube KT88      # 功率五极管（三极接法）
```

- 默认值：`12AX7`
- 实现：主线程在 Mutex 保护下替换引擎中的 Simulator 实例
- 切换中断时间：约 0ms（热切换，引擎在锁外预热后原子替换）
- 响应：`OK` 或 `ERROR unknown tube: <type>`

### model

切换放大器模型。重建整个引擎和音频流，产生约 50ms 静音。

```
model single       # 单级 12AX7 放大（已实现）
model two-stage    # 两级 12AX7，RC 耦合（占位）
model chain        # PSU + 两级 12AX7 + Tone control（占位）
```

- 默认值：`single`
- 实现：主线程 drop 当前流 → 重建 Simulator → rebuild + play 新流
- 切换中断时间：约 50ms
- 响应：`OK` 或 `ERROR <msg>`

### status

查询当前所有参数。返回 `key=value` 对，空格分隔。

请求：
```
status
```

响应：
```
bypass=off volume=-12.0 gain=-38.0 bplus=300 tube=12AX7 model=single
```

- `bypass`：`on` / `off`
- `volume`：dB
- `gain`：dB
- `bplus`：V
- `tube`：型号名
- `model`：模型名

### stop

停止 daemon 进程。无响应（socket 在进程退出前关闭）。

## 错误响应

所有命令在发生错误时返回：

```
ERROR <message>
```

常见错误：

```
ERROR unknown command: <cmd>
ERROR usage: bypass on|off
ERROR volume range: -60..+12 dB
ERROR daemon busy
ERROR daemon disconnected
```

## 实现示例

### Shell (nc)

```bash
echo "volume -6" | nc -U /tmp/danji.sock
# 响应: OK

echo "status" | nc -U /tmp/danji.sock
# 响应: bypass=off volume=-6.0 gain=-38.0 bplus=300 tube=12AX7 model=single
```

### Python

```python
import socket

def send(cmd):
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect("/tmp/danji.sock")
    s.sendall((cmd + "\n").encode())
    resp = s.recv(4096).decode().strip()
    s.close()
    return resp

print(send("status"))
print(send("model two-stage"))
```

## 设计说明

1. 协议设计为纯文本简化调试——`nc` 即可交互，无需特殊客户端
2. 每命令单次连接——daemon 不维护会话状态，每次连接独立处理
3. `model` 切换的 ~50ms 中断由主线程完成，不阻塞 socket 线程
4. `tube` 切换的预热在锁外完成，避免阻塞音频 callback
