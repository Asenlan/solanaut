# solanaut

Solana 交易人类可读解码器。输入交易签名，输出看得懂的完整交易明细——告别区块浏览器的 base64 盲文。

```
$ solanaut decode 5nS9NpJ...xK3

══════════════════════════════════════════
Transaction:  5nS9NpJ...xK3
Slot:     312,458,921
Fee:      5,000 lamports
Status:   ✓ Success
Signer:   7xK2V...sender
──────────────────────────────────────────

  #1  Transfer
  Program:    SPL Token
  Data:
    source:       7xK2V... (addr)
    destination:  9yM8F... (addr)
    amount:       1,500

  #2  swap
  Program:    Jupiter V6 (DEX Aggregator)
  Data (raw):
    discriminator:   5f81e9ab fedc2103
    hex:             ... (264 bytes total)
```

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

## 架构

```
src/
├── main.rs       # CLI 入口 (clap)
├── rpc.rs        # Solana RPC 客户端
├── decoder.rs    # 核心解码引擎
├── idl.rs        # Anchor IDL 类型 + 鉴别器
├── program_db.rs # 已知程序地址库
└── display.rs    # 终端格式化输出
```

## 技术栈

- **Rust** — 零成本抽象
- **solana-client / solana-sdk** — 官方 Solana crate (v2)
- **clap** — 声明式 CLI 参数解析
- **sha2** — Anchor 指令鉴别器计算

## 许可

MIT
