use serde::{Serialize,Deserialize};

//Usado para o get
#[derive(Clone,Serialize, Deserialize, Debug)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub age : i32,
}

//Usado para o post
#[derive(Deserialize)]
pub struct PersonReq{
    pub name: String,
    pub age : i32,
}
