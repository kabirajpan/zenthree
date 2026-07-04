use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../../file_manager/assets/"]
pub struct Assets;
