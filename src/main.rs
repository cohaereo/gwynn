mod app;
mod ui;

use eframe::wgpu;
use env_logger::Env;

use crate::app::GwynnApp;

#[macro_use]
extern crate log;

fn main() -> anyhow::Result<()> {
    dioxus_devtools::connect_subsecond();
    env_logger::Builder::from_env(
        Env::default().default_filter_or("info,wgpu_core=error,wgpu_hal=error,naga=warn"),
    )
    .init();

    info!("Starting Gwynn");

    subsecond::call(|| {
        let native_options = eframe::NativeOptions {
            window_builder: Some(Box::new(|viewport| viewport.with_inner_size((1600., 900.)))),
            // cohae: subsecond hot reload breaks with a vulkan backend, so we force DX12 on windows
            #[cfg(target_os = "windows")]
            wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
                wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(
                    eframe::egui_wgpu::WgpuSetupCreateNew {
                        instance_descriptor: eframe::wgpu::InstanceDescriptor {
                            backends: wgpu::Backends::DX12,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ),
                ..Default::default()
            },
            ..Default::default()
        };

        info!("Creating app");
        eframe::run_native(
            "Gwynn",
            native_options,
            Box::new(|cc| Ok(Box::new(GwynnApp::new(cc).unwrap()))),
        )
        .expect("Failed to start eframe");
    });

    Ok(())
}
