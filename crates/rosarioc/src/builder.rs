use crate::{CFileId, CResult, CTypeId};

pub struct Builder {
    pub result: CResult,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            result: CResult::default(),
        }
    }

    /* pub fn new_file(&mut self, name: &str) -> CFileId {
        self.result.
    } */
}
