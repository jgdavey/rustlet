use crate::settings::SmushMode;

#[derive(Debug, PartialEq)]
pub struct Text {
    /// the artwork, as lines
    pub art: Vec<String>,
    /// the unadorned text
    pub text: String,
}

impl Text {
    fn calculate_smush_amount(&self, other: &Text, smushmode: &SmushMode) -> usize {
        0
    }
    pub fn append(&mut self, other: &Text, smushmode: &SmushMode) -> Result<(), String> {
        let smushamount = self.calculate_smush_amount(other, smushmode);
        println!("{:?}", other);
        println!("{:?}", smushmode);
        Ok(())
    }
    pub fn width(&self) -> usize {
        self.art[0].len()
    }
    pub fn height(&self) -> usize {
        self.art.len()
    }
}
