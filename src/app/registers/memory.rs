use super::Register;

pub struct InMemoryRegister {
    value: Option<String>,
}

impl InMemoryRegister {
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Register for InMemoryRegister {
    fn read(&mut self) -> Option<&str> {
        self.value.as_ref().and_then(|v| Some(v.as_str()))
    }

    fn write(&mut self, value: String) {
        self.value = Some(value)
    }
}
