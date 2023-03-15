use borsh::{BorshSerialize, BorshDeserialize};
use borsh::maybestd::io::{Result, Error};
use borsh_derive::{BorshSchema};

mod serialize;
use serialize::{CustomSerialize, schema::{get_schema, TypeIterator}};

use custom_derive::CustomSerialize;

use crate::serialize::Build;

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, Debug, Clone, CustomSerialize)]
struct Person {
    uuid: u8,
    //name: String,
    //cool: bool,
    //vector: Vec<(u128, u64, String)>,
}

/*impl CustomSerialize for Person
where
    u8: CustomSerialize
{
    fn serialize(&self) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        println!("Struct");
        CustomSerialize::serialize(&self.uuid)?;
        Ok(())
    }
}*/

// End data schema

fn main() {
    //let person = Person { name: "Alison".into(), uuid: 30, cool: true, vector: vec![(100, 200, "Hello".into())] };
    //let person = Person { name: "Alison".into(), uuid: 30, cool: true };
    let person = Person { uuid: 30 };

    let tsch = get_schema::<Person>();
    let mut iter = TypeIterator::<Person>::new(&tsch);
    let mut counter: u32 = 0;
    //let mut seen: HashSet<String> = HashSet::new();
    while let Some(node) = iter.next() {
        counter += 1;
        println!("{:?}", counter);
        println!("Item {:?}", node.1);
        //println!("Parent {:?}", node.0);
        println!("");
    }
    person.try_to_custom().unwrap();
}
