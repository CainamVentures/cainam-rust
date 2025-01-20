use rig_core::{Plugin, PluginRegistrar};

mod actions;
mod providers;
mod types;

pub use actions::*;
pub use providers::*;
pub use types::*;

#[derive(Default)]
pub struct BirdeyePlugin;

impl Plugin for BirdeyePlugin {
    fn name(&self) -> &'static str {
        "birdeye"
    }

    fn description(&self) -> &'static str {
        "Birdeye plugin for token and wallet analytics on Solana"
    }

    fn register(&self, registrar: &mut dyn PluginRegistrar) {
        registrar.register_action::<TokenSearchAction>();
        registrar.register_action::<WalletSearchAction>();
    }
}

// Export plugin creation function
#[no_mangle]
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(BirdeyePlugin::default())
} 