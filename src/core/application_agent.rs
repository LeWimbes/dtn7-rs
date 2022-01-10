use std::fmt::Debug;

use bp7::EndpointID;
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
#[derive(Debug)]
pub enum ApplicationAgentEnum {
    SimpleApplicationAgent,
}

#[enum_dispatch(ApplicationAgentEnum)]
pub trait ApplicationAgent: Debug {
    fn eid(&self) -> &EndpointID;
}

#[derive(Debug, Clone)]
pub struct SimpleApplicationAgent {
    eid: EndpointID,
}

impl ApplicationAgent for SimpleApplicationAgent {
    fn eid(&self) -> &EndpointID {
        &self.eid
    }
}

impl SimpleApplicationAgent {
    pub fn with(eid: EndpointID) -> SimpleApplicationAgent {
        SimpleApplicationAgent {
            eid,
        }
    }
}
