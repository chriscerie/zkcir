use super::{expr::Expression, ident::Ident};

pub struct Assignment {
    pub ident: Ident,
    pub expression: Expression,
}
