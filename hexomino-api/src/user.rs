use uuid::Uuid;

use crate::derive_api_data;

derive_api_data! {

pub struct User {
    pub id: UserId,
    pub name: String,
}

#[derive(Copy, PartialEq, Eq, Hash, derive_more::Display)]
#[display(fmt = "{}",  _0)]
pub struct UserId(pub Uuid);

}
