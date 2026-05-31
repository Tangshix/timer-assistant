//! 生成 PNG 图标文件
//! 运行: cargo run --example generate_icon

use image::{RgbaImage, Rgba};
use std::fs;

fn main() {
    // 创建 icons 目录
    fs::create_dir_all("icons").expect("无法创建 icons 目录");
    
    let img = generate_icon_image();
    img.save("icons/app.png").expect("无法保存图标文件");
    println!("图标已生成到 icons/app.png");
}

/// 生成图标图像
fn generate_icon_image() -> RgbaImage {
    let size = 256;
    let mut img = RgbaImage::new(size, size);
    
    let center = size as f32 / 2.0;
    let radius = size as f32 / 2.0 - 8.0;
    
    for y in 0..size {
        for x in 0..size {
            let cx = x as f32 - center;
            let cy = y as f32 - center;
            let distance = (cx * cx + cy * cy).sqrt();
            
            if distance <= radius {
                // 蓝色渐变
                let gradient = (distance / radius) as f32;
                let r = (41.0 + gradient * 30.0) as u8;
                let g = (121.0 + gradient * 40.0) as u8;
                let b = (230.0 - gradient * 50.0) as u8;
                
                img.put_pixel(x, y, Rgba([r, g, b, 255]));
                
                // 时钟刻度
                let angle = (cy).atan2(cx);
                let degrees = angle.to_degrees();
                let normalized_angle = if degrees < 0.0 { degrees + 360.0 } else { degrees };
                
                if distance > radius - 12.0 && (normalized_angle % 30.0).abs() < 4.0 {
                    img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                }
                
                // 时针（10点）
                let hour_angle = (150.0_f32).to_radians();
                let hour_x = hour_angle.cos() * radius * 0.5;
                let hour_y = hour_angle.sin() * radius * 0.5;
                let dist_to_hour = ((cx - hour_x * 0.5).powi(2) + (cy - hour_y * 0.5).powi(2)).sqrt();
                if dist_to_hour < 5.0 && distance < radius * 0.6 {
                    img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                }
                
                // 分针（2点）
                let minute_angle = (60.0_f32).to_radians();
                let minute_x = minute_angle.cos() * radius * 0.75;
                let minute_y = minute_angle.sin() * radius * 0.75;
                let dist_to_minute = ((cx - minute_x * 0.5).powi(2) + (cy - minute_y * 0.5).powi(2)).sqrt();
                if dist_to_minute < 4.0 && distance < radius * 0.8 {
                    img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                }
                
                // 中心点
                if distance < 6.0 {
                    img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                }
            } else if distance <= radius + 6.0 {
                // 白色边框
                img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
            // 圆外保持透明
        }
    }
    
    img
}
