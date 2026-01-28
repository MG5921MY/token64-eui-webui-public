/*!
 * 静态资源嵌入模块
 * 
 * 使用 rust-embed 将前端文件嵌入到二进制文件中
 * 这样就不需要外部的 frontend 目录了
 */

use rust_embed::RustEmbed;

/// 嵌入的静态资源
/// 
/// 在编译时将 frontend 目录下的所有文件嵌入到二进制文件中
#[derive(RustEmbed)]
#[folder = "frontend/"]
pub struct Assets;
