use holochain_container_api::container::Container;

use neon::{context::Context, prelude::*};

use crate::config::{AgentData, DnaData, InstanceData, ScenarioConfig};

declare_types! {

    pub class JsScenarioConfig for ScenarioConfig {
        init(mut cx) {
            let instances = cx.argument::<JsArray>(0)?
                .to_vec(&mut cx)?
                .into_iter()
                .map(|v| neon_serde::from_value(&mut cx, v).expect("scenario() argument deserialization failed"))
                .collect();
            Ok(ScenarioConfig(instances))
        }

        // method run(mut cx) {
        //     let mut this = ctx.this();
        //     let func = cx.argument::<JsFunction>(0)?;
        //     func.call(&mut cx, this, args);
        // }
    }


}
