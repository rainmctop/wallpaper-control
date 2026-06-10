# Wallpaper Fetcher

一个静默运行的 Rust 壁纸自动下载和切换工具，专为 Windows 平台设计。

## 功能特点

- **静默运行**：后台运行，无用户界面，不显示控制台窗口
- **自动获取**：从硬编码的 JSON URL 获取壁纸配置
- **时间范围控制**：根据配置的起始和结束时间自动切换壁纸
- **定时检查**：每小时自动检查一次是否需要更新壁纸
- **Windows 原生支持**：使用 Windows API 设置桌面壁纸

## JSON 配置格式

工具期望从硬编码的 URL 获取如下格式的 JSON 配置：

```json
{
  "wallpapers": [
    {
      "url": "https://example.com/wallpaper1.jpg",
      "start_date": "2024-01-01T00:00:00+08:00",
      "end_date": "2024-01-31T23:59:59+08:00"
    },
    {
      "url": "https://example.com/wallpaper2.jpg",
      "start_date": "2024-02-01T00:00:00+08:00",
      "end_date": "2024-02-29T23:59:59+08:00"
    }
  ]
}
```

### 字段说明

- `url`: 壁纸图片的直接下载链接
- `start_date`: 壁纸生效的开始时间（RFC3339 格式）
- `end_date`: 壁纸失效的结束时间（RFC3339 格式）

## 使用方法

### 编译

```bash
cargo build --release
```

编译后的可执行文件位于 `target/release/wallpaper_fetcher.exe`

### 运行

直接运行可执行文件即可：

```bash
./wallpaper_fetcher.exe
```

程序将：
1. 在后台静默运行
2. 从硬编码的 URL 获取壁纸配置
3. 根据当前时间匹配对应的壁纸
4. 下载并设置为桌面壁纸
5. 每小时检查一次更新

### 开机自启动

将 `wallpaper_fetcher.exe` 添加到 Windows 启动文件夹：
1. 按 `Win + R` 打开运行对话框
2. 输入 `shell:startup` 并回车
3. 将 `wallpaper_fetcher.exe` 的快捷方式复制到打开的文件夹中

## 自定义配置

修改 `src/main.rs` 中的 `config_url` 变量来指定你的 JSON 配置 URL：

```rust
let config_url = "https://your-server.com/wallpapers.json";
```

## 注意事项

- 壁纸文件保存在用户的图片目录下的 `wallpaper_fetcher` 文件夹中
- 需要网络连接以下载壁纸
- 首次运行时会自动创建必要的目录

## 许可证

MIT License
