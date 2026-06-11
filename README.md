# Wallpaper Fetcher

一个静默运行的 Rust 壁纸自动切换工具，专为 Windows 平台设计。

## 功能特性

- **静默运行**: 无需用户交互，后台自动运行
- **定时切换**: 根据预设的时间范围自动切换壁纸
- **硬编码配置**: 壁纸 URL 和时间范围直接嵌入代码中
- **自动下载**: 从指定 URL 下载壁纸图片
- **日志记录**: 详细记录运行状态和错误信息
- **Windows 原生支持**: 使用 Windows API 设置壁纸

## 项目结构

```
wallpaper-fetcher/
├── Cargo.toml          # 项目依赖配置
├── src/
│   └── main.rs         # 主程序代码
├── .github/
│   └── workflows/
│       └── build.yml   # GitHub Actions CI/CD 配置
└── README.md           # 项目说明文档
```

## 配置说明

### 修改壁纸配置

在 `src/main.rs` 文件中找到 `WALLPAPER_CONFIG` 常量，修改其中的壁纸 URL 和时间范围：

```rust
const WALLPAPER_CONFIG: &str = r#"
[
    {
        "url": "https://example.com/wallpaper1.jpg",
        "start_time": "2024-01-01T00:00:00",
        "end_time": "2024-12-31T23:59:59"
    },
    {
        "url": "https://example.com/wallpaper2.jpg",
        "start_time": "2025-01-01T00:00:00",
        "end_time": "2025-12-31T23:59:59"
    }
]
"#;
```

### 时间格式

使用时间格式：`YYYY-MM-DDTHH:MM:SS` (ISO 8601 格式)

## 编译方法

### 本地编译 (需要 Windows 环境)

```bash
cargo build --release
```

编译后的可执行文件位于：`target/release/wallpaper-fetcher.exe`

### 使用 GitHub Actions 编译

本项目已配置 GitHub Actions，推送到仓库后会自动编译 Windows 版本：

1. 将代码推送到 GitHub 仓库
2. Actions 会自动触发构建
3. 在 Actions 页面下载编译好的 `wallpaper-fetcher.exe`

## 使用方法

1. 运行 `wallpaper-fetcher.exe`
2. 程序会在后台静默运行
3. 每小时检查一次当前时间匹配的壁纸配置
4. 自动下载并设置匹配的壁纸

### 开机自启动 (可选)

将快捷方式放入启动文件夹：
- 按 `Win + R`，输入 `shell:startup`
- 将 `wallpaper-fetcher.exe` 的快捷方式放入该文件夹

## 日志位置

日志文件保存在：
- Windows: `%LOCALAPPDATA%\wallpaper-fetcher\wallpaper-fetcher.log`

## 壁纸存储位置

下载的壁纸保存在：
- Windows: `%USERPROFILE%\Pictures\wallpapers\`

## 注意事项

- 需要网络连接以下载壁纸
- 首次运行时会创建必要的目录
- 程序会持续运行，每小时检查一次配置
- 仅支持 Windows 平台设置壁纸功能

## 依赖项

主要 Rust crate 依赖：
- `reqwest`: HTTP 客户端
- `serde`/`serde_json`: JSON 解析
- `chrono`: 日期时间处理
- `simplelog`: 日志记录
- `windows`: Windows API 绑定
- `dirs`: 获取系统目录路径

## License

MIT License
