//! 从系统剪贴板读取文件列表或图片，供附件快速粘贴。
//! 文件优先于位图，避免把资源管理器附带的预览图再存一份。

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{error::AppError, id::now_ms};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardFileDto {
    pub path: String,
    pub filename: String,
    pub ephemeral: bool,
}

fn clipboard_temp_dir() -> PathBuf {
    std::env::temp_dir().join("shixu-clipboard")
}

fn filename_of(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("attachment")
        .to_string()
}

pub fn is_clipboard_temp(path: &Path) -> bool {
    let Ok(dir) = clipboard_temp_dir().canonicalize() else {
        return false;
    };
    let Ok(path) = path.canonicalize() else {
        return false;
    };
    path.starts_with(dir)
}

#[cfg(windows)]
fn read_native() -> Result<Vec<ClipboardFileDto>, AppError> {
    use clipboard_win::{formats, get_clipboard};

    if let Ok(files) = get_clipboard::<Vec<String>, _>(formats::FileList) {
        let items: Vec<_> = files
            .into_iter()
            .map(PathBuf::from)
            .filter(|path| path.is_file())
            .map(|path| ClipboardFileDto {
                filename: filename_of(&path),
                path: path.display().to_string(),
                ephemeral: false,
            })
            .collect();
        if !items.is_empty() {
            return Ok(items);
        }
    }

    if let Some(bytes) = read_png_bytes() {
        return write_temp_png(&bytes);
    }

    if let Ok(bytes) = get_clipboard::<Vec<u8>, _>(formats::Bitmap) {
        if bytes.starts_with(b"\x89PNG") {
            return write_temp_png(&bytes);
        }
        if bytes.starts_with(b"BM") {
            if let Ok(image) = image::load_from_memory(&bytes) {
                return write_temp_image(&image);
            }
        }
    }

    Ok(vec![])
}

#[cfg(windows)]
fn read_png_bytes() -> Option<Vec<u8>> {
    use clipboard_win::{formats, get_clipboard, Format};

    for name in ["PNG", "image/png"] {
        let Some(fmt) = clipboard_win::raw::register_format(name) else {
            continue;
        };
        let raw = formats::RawData(fmt.get());
        if !raw.is_format_avail() {
            continue;
        }
        if let Ok(bytes) = get_clipboard::<Vec<u8>, _>(raw) {
            if bytes.starts_with(b"\x89PNG") {
                return Some(bytes);
            }
        }
    }
    None
}

#[cfg(windows)]
fn write_temp_image(image: &image::DynamicImage) -> Result<Vec<ClipboardFileDto>, AppError> {
    let dir = clipboard_temp_dir();
    std::fs::create_dir_all(&dir)?;
    let filename = format!("clipboard-{}.png", now_ms());
    let path = dir.join(&filename);
    image
        .save(&path)
        .map_err(|err| AppError::Invalid(format!("save clipboard image: {err}")))?;
    Ok(vec![ClipboardFileDto {
        path: path.display().to_string(),
        filename,
        ephemeral: true,
    }])
}

#[cfg(windows)]
fn write_temp_png(bytes: &[u8]) -> Result<Vec<ClipboardFileDto>, AppError> {
    let dir = clipboard_temp_dir();
    std::fs::create_dir_all(&dir)?;
    let filename = format!("clipboard-{}.png", now_ms());
    let path = dir.join(&filename);
    std::fs::write(&path, bytes)?;
    Ok(vec![ClipboardFileDto {
        path: path.display().to_string(),
        filename,
        ephemeral: true,
    }])
}

#[cfg(not(windows))]
fn read_native() -> Result<Vec<ClipboardFileDto>, AppError> {
    Ok(vec![])
}

#[tauri::command]
pub fn read_clipboard_attachments() -> Result<Vec<ClipboardFileDto>, AppError> {
    read_native()
}

#[tauri::command]
pub fn discard_clipboard_file(path: String) -> Result<(), AppError> {
    let path = PathBuf::from(path);
    if is_clipboard_temp(&path) {
        let _ = std::fs::remove_file(path);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discard_guard_rejects_paths_outside_temp() {
        let outside = Path::new("C:/Windows/notepad.exe");
        assert!(!is_clipboard_temp(outside));
    }
}
