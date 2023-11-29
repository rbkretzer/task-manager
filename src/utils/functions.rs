use image::GenericImageView;
use dioxus_desktop::tao::window:: Icon as TaoIcon;

pub(crate) fn load_icon_by_path(file_path: &str) -> Option<TaoIcon> {
    return if let Ok(image) = image::open(file_path) {
        let (width, height) = image.dimensions();
        let rgba_data: Vec<u8> = image.to_rgba8().into_raw();
        Some(TaoIcon::from_rgba(rgba_data, width, height).expect("Failed to load icon."))
    } else {
        None
    }
}