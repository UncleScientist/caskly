use crate::gestalt::*;

/// The GLK object. TODO: Insert basic usage here
#[derive(Default)]
pub struct Glk;

impl Glk {
    /// Create a new glk interface
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve capability from the gestalt system
    pub fn gestalt(&self, gestalt: Gestalt) -> GestaltResult {
        match gestalt {
            Gestalt::Version => GestaltResult::Version(0x00000705),
            Gestalt::LineInput(ch) => {
                GestaltResult::LineInput(ch as u32 >= 32 && (ch as u32) < 127)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_get_glk_version() {
        let glk = Glk::new();
        assert_eq!(
            GestaltResult::Version(0x00000705),
            glk.gestalt(Gestalt::Version)
        );
    }
}
