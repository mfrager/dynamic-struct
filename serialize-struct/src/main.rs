use borsh::{BorshSerialize, BorshDeserialize};
use borsh_derive::{BorshSchema};

mod serialize;
use serialize::{CustomSerialize, schema::{get_schema, TypeIterator}};

use custom_derive::CustomSerialize;

use crate::serialize::Build;

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, CustomSerialize, Debug, Clone)]
struct PersonInfo {
    data: String,
}

#[derive(BorshSerialize, BorshDeserialize, BorshSchema, CustomSerialize, Debug, Clone)]
struct Person {
    name: String,
    more: (PersonInfo, PersonInfo),
    //uuid: u128,
    //mode: u8,
    //new: bool,
    //fl1: f32,
    //fl2: f64,
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
    //uuid: 1234, mode: 123, name: "Bob".into(), new: false, fl1: 1.1, fl2: 2.2,
    let person = Person {
        name: "Bob".into(),
        more: (
            PersonInfo { data: "More 1".into() },
            PersonInfo { data: "More 2".into() },
        ),
    };

    let tsch = get_schema::<Person>();
    /*let mut iter = TypeIterator::<Person>::new(&tsch);
    let mut counter: u32 = 0;
    //let mut seen: HashSet<String> = HashSet::new();
    while let Some(node) = iter.next() {
        counter += 1;
        println!("{:?}", counter);
        println!("Item {:?}", node.1);
        //println!("Parent {:?}", node.0);
        println!("");
    }*/
    person.try_to_custom(&tsch).unwrap();
}
