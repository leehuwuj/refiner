//! Asset source: GPUI Kit's default icon bundle plus the extra Lucide icons
//! Refiner uses. Only the listed icons are embedded.

use std::borrow::Cow;

use gpui_kit::assets::{Assets, icon_assets};
use gpui_kit::{AssetSource, Result, SharedString};

icon_assets!(ExtraIcons, [
    ArrowLeftRight,
    Languages,
    Sparkles,
    ExternalLink,
    FolderOpen,
    RefreshCw,
    Save,
    Bot,
    Key,
    Zap,
    Monitor,
    Palette,
    MessageSquare,
    ChartColumn,
    Download,
    Square,
    Settings,
    Copy,
    Check,
    ArrowRight,
    Trash,
    FileText,
]);

pub struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = ExtraIcons.load(path)? {
            return Ok(Some(bytes));
        }
        Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut all = ExtraIcons.list(path)?;
        all.extend(Assets.list(path)?);
        Ok(all)
    }
}
