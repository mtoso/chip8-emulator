use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Cartridge {
    memory: Vec<u8>,
}

#[wasm_bindgen]
impl Cartridge {
    pub fn new(data: &[u8]) -> Cartridge {
        Cartridge {
            memory: data.to_vec()
        }
    }

    pub fn get_memory(&self) -> Vec<u8> {
        self.memory.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_instantiate_cartridge() {
        let cartridge = Cartridge::new(&[2,3,4,5]);
        assert_eq!(cartridge.get_memory().len(), 4);
    }
}