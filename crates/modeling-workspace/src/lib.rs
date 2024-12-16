#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::cast_possible_truncation)]

mod viewport;

use ::viewport::DynamicViewportPlugin;

#[derive(Debug)]
pub struct ModelingWorkspace {
    pub data_uuid: project::DataId,
}

impl workspace::Workspace for ModelingWorkspace {
    fn tools(&self) -> Vec<workspace::Toolgroup> {
        use workspace::{Action, Tool, Toolgroup};
        vec![Toolgroup {
            name: "Some Group".to_string(),
            tools: vec![Tool {
                name: "Some Tool".to_string(),
                action: Action(),
            }],
        }]
    }

    fn viewport_plugins(&self) -> Vec<DynamicViewportPlugin> {
        vec![DynamicViewportPlugin::new(
            viewport::ModelingViewportPlugin {
                data_uuid: self.data_uuid,
            }
            .into(),
        )
        .unwrap()]
    }
}
