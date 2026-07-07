use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsServicePlan {
    pub supported: bool,
    pub service_name: String,
    pub display_name: String,
    pub status: String,
    pub message: String,
}

pub trait WindowsServiceController {
    fn plan(&self) -> WindowsServicePlan;
}

#[derive(Debug, Clone, Default)]
pub struct ReservedWindowsServiceController;

impl WindowsServiceController for ReservedWindowsServiceController {
    fn plan(&self) -> WindowsServicePlan {
        WindowsServicePlan {
            supported: cfg!(windows),
            service_name: "Gate".to_string(),
            display_name: "Gate Runtime".to_string(),
            status: "reserved".to_string(),
            message: "Windows Service mode is reserved for a future release.".to_string(),
        }
    }
}
