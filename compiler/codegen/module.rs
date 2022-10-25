pub struct ModuleGenerator<'a> {
    pub(crate) _module: &'a (),
}

impl<'a> ModuleGenerator<'a> {
    pub fn new(_module: &'a ()) -> Self {
        Self { _module }
    }
}
