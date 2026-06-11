use chrono::{DateTime, Local};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct WallpaperInfo {
    url: String,
    start_date: Option<String>,
    end_date: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
}

fn main() {
    // Hardcoded server URL (JSON endpoint)
    let server_url = "https://example.com/wallpapers.json";
    
    // Run in a loop to check periodically
    loop {
        match run_once(server_url) {
            Ok(_) => {}
            Err(e) => eprintln!("Error: {}", e),
        }
        
        // Check every 5 minutes
        thread::sleep(Duration::from_secs(300));
    }
}

fn run_once(server_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Fetch wallpaper configuration from server
    let client = reqwest::blocking::Client::builder()
        .user_agent("WallpaperFetcher/1.0")
        .build()?;
    
    let response = client.get(server_url).send()?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to fetch config: {}", response.status()).into());
    }
    
    let response_text = response.text()?;
    
    // 尝试解析为不同的 JSON 结构以兼容多种服务器返回格式
    let wallpapers: Vec<WallpaperInfo> = if let Ok(list) = serde_json::from_str::<Vec<WallpaperInfo>>(&response_text) {
        // 情况 1: 直接返回数组 [{...}, {...}]
        list
    } else if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&response_text) {
        // 情况 2 & 3 & 4: 返回的是对象 {...}
        if obj.is_object() {
            // 尝试查找常见的键名 (wallpapers, data, list, items, result)
            let keys = ["wallpapers", "data", "list", "items", "result"];
            let mut found_list = None;
            
            for key in &keys {
                if let Some(val) = obj.get(*key) {
                    if val.is_array() {
                        found_list = Some(val.clone());
                        break;
                    }
                }
            }

            if let Some(array_val) = found_list {
                // 情况 2: 找到了嵌套的数组 {"wallpapers": [...]}
                serde_json::from_value(array_val)?
            } else {
                // 情况 4: 对象本身就是单张壁纸配置，将其包装为数组
                vec![serde_json::from_value(obj)?]
            }
        } else {
            return Err("Invalid JSON format: expected an object or array".into());
        }
    } else {
        return Err("Failed to parse JSON response".into());
    };
    
    if wallpapers.is_empty() {
        println!("No wallpapers found in configuration.");
        return Ok(());
    }
    
    // Get current date and time
    let now: DateTime<Local> = Local::now();
    let current_date = now.format("%Y-%m-%d").to_string();
    let current_time = now.format("%H:%M").to_string();
    
    // Find the appropriate wallpaper
    let mut selected_wallpaper: Option<&WallpaperInfo> = None;
    
    for wallpaper in &wallpapers {
        if is_wallpaper_active(wallpaper, &current_date, &current_time) {
            selected_wallpaper = Some(wallpaper);
            break;
        }
    }
    
    if let Some(wallpaper) = selected_wallpaper {
        // Download the wallpaper
        let download_path = download_wallpaper(&client, &wallpaper.url)?;
        
        // Set as desktop background
        set_wallpaper(&download_path)?;
        
        println!("Wallpaper updated successfully: {}", wallpaper.url);
    } else {
        println!("No active wallpaper found for current time");
    }
    
    Ok(())
}

fn is_wallpaper_active(wallpaper: &WallpaperInfo, current_date: &str, current_time: &str) -> bool {
    // Check date range
    if let Some(start_date) = &wallpaper.start_date {
        if current_date < start_date.as_str() {
            return false;
        }
    }
    
    if let Some(end_date) = &wallpaper.end_date {
        if current_date > end_date.as_str() {
            return false;
        }
    }
    
    // Check time range
    if let Some(start_time) = &wallpaper.start_time {
        if current_time < start_time.as_str() {
            return false;
        }
    }
    
    if let Some(end_time) = &wallpaper.end_time {
        if current_time > end_time.as_str() {
            return false;
        }
    }
    
    true
}

fn download_wallpaper(client: &reqwest::blocking::Client, url: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Get the pictures directory
    let pictures_dir = dirs::picture_dir()
        .unwrap_or_else(|| PathBuf::from("./pictures"));
    
    fs::create_dir_all(&pictures_dir)?;
    
    // Generate filename from URL
    let filename = url.split('/').last().unwrap_or("wallpaper.jpg");
    let download_path = pictures_dir.join(filename);
    
    // Download the file
    let response = client.get(url).send()?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download wallpaper: {}", response.status()).into());
    }
    
    let bytes = response.bytes()?;
    fs::write(&download_path, bytes)?;
    
    Ok(download_path)
}

#[cfg(target_os = "windows")]
fn set_wallpaper(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
    use windows::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE};
    
    unsafe {
        // Initialize COM
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;
        
        // Convert path to wide string
        let wide_path: Vec<u16> = OsStr::new(path.as_os_str())
            .encode_wide()
            .chain(Some(0))
            .collect();
        
        // Set the wallpaper
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(wide_path.as_ptr() as *mut core::ffi::c_void),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )?;
        
        CoUninitialize();
    }
    
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn set_wallpaper(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Placeholder for non-Windows platforms
    println!("Setting wallpaper on non-Windows platform (not implemented): {:?}", path);
    Ok(())
}
