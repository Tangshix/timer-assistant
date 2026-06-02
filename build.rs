fn main() {
    // 编译 Slint UI
    slint_build::compile("ui/timer_app.slint").unwrap();

    // Windows: 将 PNG 转换为 ICO 并嵌入 exe 资源（设置任务栏图标）
    #[cfg(windows)]
    {
        let png_path = "icons/app.png";
        let ico_path = "target/app.ico";
        let rc_path = "target/app.rc";

        if let Ok(img) = image::open(png_path) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let ico_file = std::fs::File::create(ico_path).unwrap();
            image::codecs::ico::IcoEncoder::new(ico_file)
                .write_image(rgba.as_raw(), w, h, image::ColorType::Rgba8)
                .unwrap();

            std::fs::write(rc_path, format!("1 ICON \"{}\"", ico_path)).unwrap();
            embed_resource::compile(rc_path, embed_resource::None);
        }
    }
}
