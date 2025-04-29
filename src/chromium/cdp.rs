use tokio::task;

use crate::chromium::chromium_manager::ChromiumProfile;

struct CdpMit {
    
}
impl CdpMit {
    fn new(&self) -> CdpMit {
        Self {
            
        }
    }
    
    pub fn start_cdp_mit(&self, profile: &ChromiumProfile) {
        task::spawn(async move {
            
        });
    }
    
}
