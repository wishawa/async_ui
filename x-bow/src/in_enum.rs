pub struct InEnumYes;
pub struct InEnumNo;
pub trait IsInEnum {}
impl IsInEnum for InEnumYes {}
impl IsInEnum for InEnumNo {}
