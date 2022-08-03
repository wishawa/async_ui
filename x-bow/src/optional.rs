pub struct OptionalYes;
pub struct OptionalNo;
pub trait IsOptional {}
impl IsOptional for OptionalYes {}
impl IsOptional for OptionalNo {}
