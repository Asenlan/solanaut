[English](README.md) | [中文](README_CN.md)

---

# solanaut

Solana 交易人类可读解码器。输入交易签名，输出看得懂的完整交易明细——告别区块浏览器的 base64 盲文。

## 功能

- **25+ 已知程序识别** — Jupiter, Orca, Raydium, Pump.fun, Jito, Metaplex, Magic Eden 等
- **解析指令支持** — SPL Token / System Program 指令被 RPC 节点预解析
- **Anchor 鉴别器匹配** — 通过 sha256("global:<name>") 前 8 字节匹配指令名
- **Borsh 字段解码** — 解码常见 Anchor 指令参数（u8-u128, bool, Pubkey, String）
- **JSON 输出** — `--json` 参数支持管道到 `jq` 或脚本
- **彩色终端** — 地址、金额、状态分颜色显示

## 安装

```bash
cargo install --git https://github.com/user/solanaut
```

## 使用

```bash
solanaut decode <SIGNATURE>          # 解码交易
solanaut decode <SIGNATURE> --json   # JSON 输出
solanaut decode <SIGNATURE> --rpc <URL>  # 自定义 RPC
solanaut programs                    # 列出已知程序
```

## 许可

MIT
