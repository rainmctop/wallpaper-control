use chrono::{DateTime, Local, NaiveDateTime};
use log::{info, error, warn};
use reqwest::blocking::Client;
use serde::Deserialize;
use simplelog::{WriteLogger, Config};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, RECT};
#[cfg(target_os = "windows")]
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Shell::{IActiveDesktop, ActiveDesktop};

// 硬编码的 JSON 配置，包含壁纸 URL 和时间范围
const WALLPAPER_CONFIG: &str = r#"
[
    {
        "url": "https://picsum.photos/1920/1080?random=1",
        "start_time": "2024-01-01T00:00:00",
        "end_time": "2024-12-31T23:59:59"
    },
    {
        "url": "https://picsum.photos/1920/1080?random=2",
        "start_time": "2025-01-01T00:00:00",
        "end_time": "2025-12-31T23:59:59"
    }
]
"#;

#[derive(Debug, Deserialize)]
struct WallpaperConfig {
    url: String,
    start_time: String,
    end_time: String,
}

fn parse_datetime(dt_str: &str) -> Option<DateTime<Local>> {
    NaiveDateTime::parse_from_str(dt_str, "%Y-%m-%dT%H:%M:%S")
        .ok()
        .map(|naive| naive.and_local_timezone(Local).unwrap())
}

fn get_current_wallpaper_config() -> Option<WallpaperConfig> {
    let configs: Vec<WallpaperConfig> = match serde_json::from_str(WALLPAPER_CONFIG) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to parse wallpaper config: {}", e);
            return None;
        }
    };

    let now = Local::now();
    
    for config in configs {
        if let (Some(start), Some(end)) = (
            parse_datetime(&config.start_time),
            parse_datetime(&config.end_time),
        ) {
            if now >= start && now <= end {
                info!(
                    "Found valid wallpaper config for period: {} to {}",
                    config.start_time, config.end_time
                );
                return Some(config);
            }
        } else {
            warn!("Invalid datetime format in config: {:?}", config);
        }
    }

    None
}

fn download_wallpaper(client: &Client, url: &str, save_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    info!("Downloading wallpaper from: {}", url);
    let response = client.get(url).send()?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download: {}", response.status()).into());
    }

    let bytes = response.bytes()?;
    let mut file = File::create(save_path)?;
    file.write_all(&bytes)?;
    
    info!("Wallpaper saved to: {:?}", save_path);
    Ok(())
}

#[cfg(target_os = "windows")]
fn set_wallpaper_windows(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;
        
        let desktop: IActiveDesktop = ActiveDesktop.new()?;
        
        // Convert PathBuf to wide string for Windows API
        let wide_path: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        
        let options = windows::Win32::UI::Shell::ADW_SETWALLPAPER;
        desktop.SetWallpaper(wide_path.as_ptr(), options)?;
        desktop.ApplyChanges(windows::Win32::UI::Shell::ADAP_REFRESHINIFILE | windows::Win32::UI::Shell::ADAP_UPDATEIMAGE)?;
    }
    
    info!("Wallpaper set successfully on Windows");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn set_wallpaper_windows(_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    warn!("Setting wallpaper is only supported on Windows");
    Ok(())
}

fn get_wallpaper_dir() -> PathBuf {
    let mut path = dirs::picture_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("wallpapers");
    
    if !path.exists() {
        fs::create_dir_all(&path).expect("Failed to create wallpaper directory");
    }
    
    path
}

fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("wallpaper-fetcher");
    
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    
    path.push("wallpaper-fetcher.log");
    
    let config = Config::default();
    WriteLogger::init(log::LevelFilter::Info, config, File::create(path)?)?;
    
    Ok(())
}

fn main() {
    // 初始化日志
    if let Err(e) = init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        return;
    }

    info!("Wallpaper Fetcher started");

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    loop {
        match get_current_wallpaper_config() {
            Some(config) => {
                let wallpaper_dir = get_wallpaper_dir();
                let filename = format!(
                    "wallpaper_{}.jpg",
                    Local::now().format("%Y%m%d_%H%M%S")
                );
                let save_path = wallpaper_dir.join(&filename);

                match download_wallpaper(&client, &config.url, &save_path) {
                    Ok(_) => {
                        if let Err(e) = set_wallpaper_windows(&save_path) {
                            error!("Failed to set wallpaper: {}", e);
                        } else {
                            info!("Successfully changed wallpaper");
                        }
                    }
                    Err(e) => {
                        error!("Failed to download wallpaper: {}", e);
                    }
                }
            }
            None => {
                warn!("No valid wallpaper configuration found for current time");
            }
        }

        // 每小时检查一次
        info!("Sleeping for 1 hour...");
        thread::sleep(Duration::from_secs(3600));
    }
}
