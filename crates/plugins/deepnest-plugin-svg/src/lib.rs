
use deepnest_common_plugin::{Plugin, Capability, API_VERSION};

struct SvgExport;
impl Capability for SvgExport {
    fn call(&self, input: &str) -> String {
        format!("<svg>{}</svg>", input)
    }

    fn name(&self) -> &'static str {
        "export_svg"
    }
}

struct SvgPlugin;
impl Plugin for SvgPlugin {
    fn name(&self) -> &'static str {
        "SVGPlugin"
    }

    fn capabilities(&self) -> Vec<Box<dyn Capability>> {
        vec![Box::new(SvgExport)]
    }
}

#[no_mangle]
pub extern "C" fn get_api_version() -> u32 {
    API_VERSION
}

#[no_mangle]
pub extern "C" fn register_plugin(reg: &mut dyn FnMut(Box<dyn Plugin>)) {
    reg(Box::new(SvgPlugin));
}