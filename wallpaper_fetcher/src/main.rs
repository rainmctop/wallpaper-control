use chrono::{DateTime, Local};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
struct WallpaperConfig {
    url: String,
    start_date: String,
    end_date: String,
}

#[derive(Debug, Deserialize)]
struct WallpaperResponse {
    wallpapers: Vec<WallpaperConfig>,
}

/// 检查当前时间是否在指定的起始和结束时间范围内
fn is_within_date_range(start_date: &str, end_date: &str) -> bool {
    let now = Local::now();
    
    let start = match DateTime::parse_from_rfc3339(start_date) {
        Ok(dt) => dt.with_timezone(&Local),
        Err(_) => {
            eprintln!("无法解析开始日期: {}", start_date);
            return false;
        }
    };
    
    let end = match DateTime::parse_from_rfc3339(end_date) {
        Ok(dt) => dt.with_timezone(&Local),
        Err(_) => {
            eprintln!("无法解析结束日期: {}", end_date);
            return false;
        }
    };
    
    now >= start && now <= end
}

/// 下载壁纸文件
fn download_wallpaper(client: &Client, url: &str, save_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let response = client.get(url).send()?;
    let bytes = response.bytes()?;
    fs::write(save_path, bytes)?;
    Ok(())
}

/// 设置 Windows 壁纸
#[cfg(target_os = "windows")]
fn set_wallpaper(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    
    let wide_path: Vec<u16> = path.as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    
    unsafe {
        // 使用 SystemParametersInfoW 设置壁纸
        let result = windows::Win32::UI::Shell::SystemParametersInfoW(
            windows::Win32::UI::Shell::SPI_SETDESKWALLPAPER,
            0,
            Some(wide_path.as_ptr() as *const _),
            windows::Win32::UI::Shell::SPIF_UPDATEINIFILE | windows::Win32::UI::Shell::SPIF_SENDCHANGE,
        );
        
        if result.0 == 0 {
            return Err("设置壁纸失败".into());
        }
    }
    
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn set_wallpaper(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 非 Windows 平台的占位实现
    println!("[模拟] 设置壁纸为: {:?}", path);
    Ok(())
}

/// 获取图片保存目录
fn get_wallpaper_dir() -> PathBuf {
    let mut dir = dirs::picture_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("wallpaper_fetcher");
    fs::create_dir_all(&dir).expect("无法创建壁纸目录");
    dir
}

/// 主逻辑：检查并更新壁纸
fn check_and_update_wallpaper() -> Result<(), Box<dyn std::error::Error>> {
    // 硬编码的 JSON URL（示例 URL，实际使用时请替换）
    let config_url = "https://example.com/wallpapers.json";
    
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    
    // 获取壁纸配置列表
    let response = match client.get(config_url).send() {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("获取壁纸配置失败: {}", e);
            return Err(Box::new(e));
        }
    };
    
    let config: WallpaperResponse = response.json()?;
    
    let wallpaper_dir = get_wallpaper_dir();
    
    // 遍历所有壁纸配置，找到当前时间范围内的壁纸
    for wallpaper in &config.wallpapers {
        if is_within_date_range(&wallpaper.start_date, &wallpaper.end_date) {
            let filename = format!(
                "wallpaper_{}.jpg",
                chrono::Local::now().format("%Y%m%d_%H%M%S")
            );
            let save_path = wallpaper_dir.join(&filename);
            
            // 如果文件已存在且是今天下载的，跳过
            if save_path.exists() {
                // 检查是否需要重新下载（可选逻辑）
                set_wallpaper(&save_path)?;
                println!("已设置现有壁纸: {:?}", save_path);
                return Ok(());
            }
            
            // 下载新壁纸
            println!("正在下载壁纸: {}", wallpaper.url);
            download_wallpaper(&client, &wallpaper.url, &save_path)?;
            
            // 设置壁纸
            set_wallpaper(&save_path)?;
            println!("成功设置新壁纸: {:?}", save_path);
            return Ok(());
        }
    }
    
    println!("未找到当前时间范围内的壁纸配置");
    Ok(())
}

fn main() {
    // 静默运行，不显示窗口
    #[cfg(target_os = "windows")]
    {
        use std::ptr;
        unsafe {
            // 隐藏控制台窗口（仅 Windows）
            let hwnd = windows::Win32::Foundation::HWND(ptr::null_mut());
            windows::Win32::UI::WindowsAndMessaging::ShowWindow(hwnd, windows::Win32::UI::WindowsAndMessaging::SW_HIDE);
        }
    }
    
    loop {
        match check_and_update_wallpaper() {
            Ok(_) => {},
            Err(e) => eprintln!("错误: {}", e),
        }
        
        // 每小时检查一次
        thread::sleep(Duration::from_secs(3600));
    }
}
